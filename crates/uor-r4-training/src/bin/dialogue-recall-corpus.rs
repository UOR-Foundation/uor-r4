//! Step 6: a natural-dialogue recall curriculum (#820).
//!
//! ```text
//! dialogue-recall-corpus generate out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   [panels=data/panels[,DIR...]] [seed=1] (dialogues=N | token_budget=N) \
//!   [dev_seed=1000003] [dev_dialogues=300] [protocol=2] [context=384] \
//!   [samples=200] [source_commit=SHA]
//! dialogue-recall-corpus leak out=NEW_REPORT_ROOT store=STORE_DIR tokenizer=TOKENIZER.json \
//!   panels=DIR[,DIR...]
//! ```
//!
//! `generate` writes multi-turn user/assistant dialogues in plain conversational
//! English whose last assistant turns can only be right by retrieving a value
//! stated earlier in the same conversation. It writes two prepared dialogue
//! splits in the format `dialogue-train` and `mix-chat-corpus` read
//! (`uor-r4-chat-corpus/v1`: `tokens.u16`, `response_mask.u8`, `manifest.json`):
//! `OUT/train/` (from `seed`, until `dialogues` or `token_budget`) and a small
//! monitoring split `OUT/dev/` (from `dev_seed`). Beside them: `OUT/leak.json`,
//! the leak report against the frozen panels, and `OUT/generator.json`, the
//! counts per category, seeds, source commit and SHA-256 of every file. The
//! root is claimed before anything is written and sealed at the end.
//!
//! What a dialogue contains (one seeded draw per dialogue, deterministic):
//!
//! - Facts stated in user turns (and, in `assistant_stated`, chosen by the
//!   assistant and accepted by the user), usually two keys of the same type in
//!   one sentence ("the turtle is Shelby and the goldfish is Flash"), so the
//!   typical error is the other key's value.
//! - 0 to 6 distractor exchanges (small talk and unrelated questions with
//!   ordinary replies), an optional fact turn from a second relation, and then
//!   one to three questions. Every question's assistant reply is a scored
//!   response; so are the acknowledgements and distractor replies (the trainer
//!   scores every assistant turn).
//! - Question categories: `binding` (ask one of several same-type keys),
//!   `update` (a later turn corrects one value; the latest value wins, and the
//!   unchanged key keeps its value), `reverse` (who/which key holds a value),
//!   `assistant_stated`, `cross_relation` (one key in each of two relations) and
//!   `abstain` (a key that was never given a value: the reply says so and names
//!   no stated value; about 15% of questions).
//! - Answers vary: the relation's own sentence, a bare value, "You said ...",
//!   "You changed it to ...", "I suggested ...".
//!
//! Disjointness from the frozen panels (`panels=`, default `data/panels`):
//! every `*.json` (all strings except `id` and `category`) and `*.tsv` (every
//! field after the id) is read. The value pools exclude every content word of
//! every panel string and check field, so no recall value shares a word stem
//! with a panel. The values of the panels' recall rows (the `terms` and
//! `forbid` columns of rows whose history is `recall`, including the closed
//! classes of colours, weekdays and the numbers two to twelve) appear nowhere in
//! the generated text. No generated turn equals a panel string (words compared
//! lowercased, punctuation ignored) and no generated turn shares a word 6-gram
//! with one; a draw that would is redrawn and counted. Relation key words
//! (turtle, Grandpa, ...) may coincide with panel words: at least `0.6` of the
//! dialogues use only keys that share no content word with any panel, and the
//! report lists every shared key and frame word with its count.
//!
//! `leak.json` is computed from the written token stores, decoded back to
//! text, not from the generator's own records (except the value and key word
//! lists, which only the records carry). `leak` re-runs that decoded check on
//! any prepared store against any panel directories (for example the local
//! ladder panels).
//!
//! Scope: this is training data. It is not an evaluation, and the development
//! split shares the generator's pools and templates; the frozen panels remain
//! the held-out measurement.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::stack_dialogue::{episode_contract_for, DialogueSplit};
use uor_r4_training::stack_tracking::Rng;

type Result<T> = std::result::Result<T, String>;

const SCHEMA: &str = "uor-r4.dialogue-recall-corpus/1";
const LEAK_SCHEMA: &str = "uor-r4.dialogue-recall-leak/1";
/// Word n-gram length of the panel overlap check.
const NGRAM: usize = 6;
/// Share of dialogues whose keys share no content word with any panel.
const STRICT_KEY_SHARE: f64 = 0.6;
/// Primary question category shares (the follow-ups below add binding and
/// abstention questions; the realised per-question shares are reported).
const PRIMARY_SHARES: [(Category, f64); 6] = [
    (Category::Binding, 0.38),
    (Category::Update, 0.16),
    (Category::Reverse, 0.12),
    (Category::AssistantStated, 0.09),
    (Category::CrossRelation, 0.12),
    (Category::Abstain, 0.13),
];
/// A follow-up question is an abstention with this probability.
const FOLLOWUP_ABSTAIN: f64 = 0.15;
const MAX_DISTRACTORS: usize = 6;

// ---------------------------------------------------------------------------
// Text normalisation shared by the panel reader, the generator and the checks.

#[rustfmt::skip]
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "if", "then", "so", "of", "to", "in", "on", "at", "by",
    "for", "with", "from", "up", "down", "out", "over", "under", "about", "into", "onto", "off",
    "as", "is", "are", "was", "were", "be", "been", "being", "am", "do", "does", "did", "done",
    "have", "has", "had", "i", "me", "my", "mine", "we", "us", "our", "ours", "you", "your",
    "yours", "he", "him", "his", "she", "her", "hers", "it", "its", "they", "them", "their",
    "theirs", "this", "that", "these", "those", "what", "which", "who", "whom", "whose", "where",
    "when", "why", "how", "there", "here", "not", "no", "yes", "all", "any", "some", "each",
    "every", "both", "too", "very", "just", "also", "can", "could", "will", "would", "shall",
    "should", "may", "might", "must", "let", "lets", "oh", "okay", "ok", "please", "than", "now",
    "again", "still", "only", "own", "same", "other", "another", "such", "more", "most", "much",
    "many", "few", "one", "get", "got", "go", "going", "went", "said", "say", "tell", "told",
    "know", "dont", "im", "ive", "youre", "thats", "whats", "didnt", "doesnt", "isnt", "wasnt",
    "cant", "wont", "ill", "id", "theyre", "hasnt", "havent", "theres", "heres", "whos", "its",
    "after", "before", "while", "because", "like", "really", "well", "way", "yet", "ever", "never",
    "something", "anything", "nothing", "please", "thanks", "thank", "hi", "hello", "hey",
];

/// Lowercased words: letters and digits; a possessive "'s" is removed and
/// other apostrophes are dropped inside a word; every other character is a
/// separator.
fn words(text: &str) -> Vec<String> {
    fn finish(current: &mut String, out: &mut Vec<String>) {
        let mut word = std::mem::take(current);
        if word.ends_with("'s") {
            word.truncate(word.len() - 2);
        }
        let word: String = word.chars().filter(|&c| c != '\'').collect();
        if !word.is_empty() {
            out.push(word);
        }
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch == '\'' || ch == '\u{2019}' {
            if !current.is_empty() {
                current.push('\'');
            }
        } else if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else {
            finish(&mut current, &mut out);
        }
    }
    finish(&mut current, &mut out);
    out
}

/// A light plural/possessive stem, applied to both sides of every comparison.
fn stem(word: &str) -> String {
    let n = word.len();
    if !word.is_ascii() {
        return word.to_owned();
    }
    if n > 4 && word.ends_with("ies") {
        return format!("{}y", &word[..n - 3]);
    }
    if n > 4
        && ["ches", "shes", "sses", "xes", "zes", "oes"]
            .iter()
            .any(|s| word.ends_with(s))
    {
        return word[..n - 2].to_owned();
    }
    if n > 3
        && word.ends_with('s')
        && !word.ends_with("ss")
        && !word.ends_with("us")
        && !word.ends_with("is")
    {
        return word[..n - 1].to_owned();
    }
    word.to_owned()
}

fn is_stop(word: &str) -> bool {
    STOPWORDS.contains(&word)
}

/// Stems of the content (non-stopword, longer than one character) words.
fn content_stems(text: &str) -> BTreeSet<String> {
    words(text)
        .into_iter()
        .filter(|w| w.chars().count() > 1 && !is_stop(w))
        .map(|w| stem(&w))
        .collect()
}

fn normalized(text: &str) -> String {
    words(text).join(" ")
}

fn grams(text: &str) -> Vec<String> {
    let w = words(text);
    if w.len() < NGRAM {
        return Vec::new();
    }
    w.windows(NGRAM).map(|g| g.join(" ")).collect()
}

fn cap(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Panels.

struct Panel {
    /// (path, sha256) of every file read.
    files: Vec<(String, String)>,
    /// Normalised whole strings.
    turns: HashSet<String>,
    /// Word 6-grams of every string.
    grams: HashSet<String>,
    /// Content stems of every string and check field.
    content: BTreeSet<String>,
    /// Content stems of the recall rows' `terms` and `forbid` columns: these
    /// appear nowhere in the generated text.
    strict: BTreeSet<String>,
    /// Content stems of the recall rows' `keys` column.
    keys: BTreeSet<String>,
    strings: usize,
}

impl Panel {
    fn add_string(&mut self, text: &str) {
        let norm = normalized(text);
        if norm.is_empty() {
            return;
        }
        self.strings += 1;
        self.turns.insert(norm);
        for g in grams(text) {
            self.grams.insert(g);
        }
        self.content.extend(content_stems(text));
    }

    fn load(dirs: &[PathBuf]) -> Result<Self> {
        let mut panel = Panel {
            files: Vec::new(),
            turns: HashSet::new(),
            grams: HashSet::new(),
            content: BTreeSet::new(),
            strict: BTreeSet::new(),
            keys: BTreeSet::new(),
            strings: 0,
        };
        for dir in dirs {
            let mut entries: Vec<PathBuf> = fs::read_dir(dir)
                .map_err(|e| format!("{}: {e}", dir.display()))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    matches!(
                        p.extension().and_then(|x| x.to_str()),
                        Some("json") | Some("tsv")
                    )
                })
                .collect();
            entries.sort();
            for path in entries {
                let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let sha = uor_r4_training::sha256_file(&path).map_err(|e| e.to_string())?;
                panel.files.push((path.display().to_string(), sha));
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_owned();
                if name.ends_with(".json") {
                    let value: Value = serde_json::from_slice(&bytes)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    let mut strings = Vec::new();
                    collect_strings(&value, &mut strings);
                    for s in strings {
                        panel.add_string(&s);
                    }
                } else {
                    let text =
                        String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
                    let checks = name.contains("checks");
                    for line in text.lines() {
                        if line.starts_with('#') || line.trim().is_empty() {
                            continue;
                        }
                        let fields: Vec<&str> = line.split('\t').collect();
                        for (column, field) in fields.iter().enumerate().skip(1) {
                            if checks && column < 3 {
                                continue; // kind, history
                            }
                            for phrase in field.split('|') {
                                if phrase.trim() != "-" {
                                    panel.add_string(phrase);
                                }
                            }
                        }
                        if checks && fields.get(2).map(|h| h.trim()) == Some("recall") {
                            for column in [3usize, 4] {
                                if let Some(field) = fields.get(column) {
                                    for phrase in field.split('|') {
                                        panel.strict.extend(content_stems(phrase));
                                    }
                                }
                            }
                            if let Some(field) = fields.get(5) {
                                for phrase in field.split('|') {
                                    panel.keys.extend(content_stems(phrase));
                                }
                            }
                        }
                    }
                }
            }
        }
        if panel.files.is_empty() {
            return Err("no panel files (*.json, *.tsv) were found under panels=".into());
        }
        Ok(panel)
    }

    /// Whether a text shares a stem with the panels' content words.
    fn shares_content(&self, text: &str) -> bool {
        content_stems(text).iter().any(|s| self.content.contains(s))
    }

    fn has_strict(&self, text: &str) -> bool {
        content_stems(text).iter().any(|s| self.strict.contains(s))
    }

    /// Whether a generated turn leaks: equal to a panel string, sharing a
    /// 6-gram with one, or holding a recall-row value word.
    fn turn_leak(&self, text: &str) -> Option<&'static str> {
        if self.turns.contains(&normalized(text)) {
            return Some("panel_turn");
        }
        if grams(text).iter().any(|g| self.grams.contains(g)) {
            return Some("panel_6gram");
        }
        if self.has_strict(text) {
            return Some("panel_recall_value_word");
        }
        None
    }
}

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| collect_strings(v, out)),
        Value::Object(map) => {
            for (key, v) in map {
                if key != "id" && key != "category" {
                    collect_strings(v, out);
                }
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Pools.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Vc {
    PetName,
    PersonName,
    Food,
    Dish,
    Drink,
    Object,
    Age,
    Job,
    Place,
    Street,
    Instrument,
    Spot,
    Color,
    Time,
    Date,
    Plant,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kc {
    Pets,
    People,
    Roles,
    Items,
    Paintables,
    Appointments,
    Events,
    Beds,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Art {
    None,
    Indef,
    The,
}

#[rustfmt::skip]
const PERSON_NAMES: &[&str] = &[
    "Aiden", "Amara", "Anika", "Arjun", "Astrid", "Bastian", "Beatrix", "Bodhi", "Calla", "Cedric",
    "Celeste", "Dario", "Delphine", "Desmond", "Elio", "Elodie", "Emeka", "Esme", "Ezra", "Farah",
    "Felix", "Fiona", "Gideon", "Greta", "Hamish", "Hana", "Idris", "Ines", "Isak", "Jago",
    "Kaito", "Keziah", "Lars", "Leilani", "Linus", "Lorcan", "Mabel", "Marisol", "Matteo",
    "Mireille", "Nadia", "Nico", "Niamh", "Odette", "Oren", "Orla", "Otis", "Paloma", "Quentin",
    "Rafferty", "Rhea", "Rowan", "Sabine", "Selma", "Silas", "Soraya", "Stellan", "Tamsin",
    "Teodor", "Thea", "Tobias", "Ulla", "Valentina", "Vikram", "Wren", "Xavier", "Yara", "Yusuf",
    "Zelda", "Zubair", "Anouk", "Bram", "Casimir", "Dagny", "Emrys", "Fenna", "Gulnara", "Henrik",
    "Ilse", "Joaquin", "Kalinda", "Leopold", "Maelis", "Nuno", "Oksana", "Pilar", "Ragnar",
    "Signe", "Tariq", "Ugo", "Vesna", "Wilhelmina", "Ximena", "Yoshiro", "Zainab", "Akosua",
    "Bilal", "Chiara", "Dmitri", "Eamon", "Freya", "Gwendolyn", "Hugo", "Imogen", "Jovan",
    "Katarina", "Lucius", "Magnus", "Nell", "Osric", "Petra", "Rosamund", "Sven", "Tova",
    "Umberto", "Viggo", "Winifred", "Yannick", "Zora", "Ayesha", "Benedikt", "Clementine",
    "Dorian", "Elspeth", "Florian", "Ingrid", "Jasper", "Kenji", "Lavinia", "Marguerite",
    "Nikolai", "Ottilie", "Rufus", "Saoirse", "Thaddeus", "Ursula", "Vivienne", "Wolfgang",
    "Adaeze", "Bjorn", "Cosima", "Dov", "Eilidh", "Faisal", "Gaspard", "Halima", "Ignatius",
    "Jessamy", "Kwame", "Liesel", "Mateus", "Nkechi", "Obadiah", "Priyanka", "Radu", "Sunniva",
    "Thandiwe", "Uriel", "Vashti", "Wendell", "Xiomara", "Yevgenia", "Zephyr",
];

#[rustfmt::skip]
const PET_NAMES: &[&str] = &[
    "Noodle", "Sprocket", "Bramble", "Pudding", "Clover", "Marbles", "Gizmo", "Tofu", "Nugget",
    "Ziggy", "Mochi", "Paprika", "Domino", "Truffle", "Bandit", "Comet", "Fizz", "Gumdrop",
    "Hopscotch", "Jellybean", "Lentil", "Nimbus", "Pistachio", "Rascal", "Saffron", "Tinsel",
    "Velvet", "Yeti", "Zigzag", "Acorn", "Button", "Cinnamon", "Dumpling", "Fudge", "Inky",
    "Kipper", "Licorice", "Maple", "Nutmeg", "Rocket", "Sesame", "Toffee", "Crumpet", "Sprout",
    "Scout", "Tango", "Rumble", "Bubbles", "Cricket", "Figaro", "Goose", "Hobbes", "Igloo", "Jinx",
    "Marshmallow", "Nacho", "Quill", "Ripple", "Thimble", "Waldo", "Yoyo", "Biscotti", "Doodle",
    "Echo", "Fable", "Gnocchi", "Jalapeno", "Kazoo", "Lollipop", "Meatball", "Napkin", "Pretzel",
    "Quasar", "Raisin", "Squiggle", "Turnip", "Wobble", "Zucchini",
];

#[rustfmt::skip]
const FOODS: &[&str] = &[
    "figs", "leeks", "kale", "apricots", "radishes", "quinces", "lentils", "chestnuts", "cherries",
    "papayas", "turnips", "parsnips", "artichokes", "asparagus", "beets", "cabbages", "celery",
    "cucumbers", "eggplants", "fennel", "garlic", "grapefruit", "guavas", "kiwis", "lemons",
    "limes", "lychees", "melons", "mushrooms", "nectarines", "okra", "olives", "onions", "peaches",
    "pineapples", "pomegranates", "potatoes", "pumpkins", "raspberries", "rhubarb", "shallots",
    "spinach", "squash", "strawberries", "tangerines", "walnuts", "almonds", "hazelnuts", "pecans",
    "cashews", "peanuts", "prunes", "raisins", "blueberries", "blackberries", "cranberries",
    "gooseberries", "clementines", "persimmons", "plantains", "yams", "zucchini", "bagels",
    "croissants", "pretzels", "scones", "tortillas", "cheddar", "brie", "mozzarella", "feta",
    "salami", "sardines", "salmon", "shrimp", "tofu", "honey", "maple syrup", "oat milk", "rice",
    "couscous", "quinoa", "noodles", "samosas", "hummus", "sauerkraut", "kimchi", "granola",
    "pistachios", "dried mangoes", "sweet potatoes", "brussels sprouts", "watercress", "kohlrabi",
    "endives", "jackfruit", "dragon fruit",
];

#[rustfmt::skip]
const DISHES: &[&str] = &[
    "lasagna", "paella", "ramen", "risotto", "curry", "tacos", "sushi", "goulash", "moussaka",
    "pierogi", "biryani", "gnocchi", "chowder", "enchiladas", "jollof rice", "pad thai",
    "shakshuka", "borscht", "gumbo", "tagine", "ratatouille", "pho", "burritos", "quesadillas",
    "crepes", "dumpling soup", "fish stew", "mac and cheese", "chili", "kebabs", "falafel",
    "pozole", "laksa", "bibimbap", "carbonara", "pesto pasta", "fried rice", "lentil soup",
    "stuffed peppers", "shepherd's pie", "fajitas", "katsu curry", "mapo tofu", "spanakopita",
    "jambalaya", "ceviche", "lamb stew", "minestrone", "dal", "tamales",
];

#[rustfmt::skip]
const DRINKS: &[&str] = &[
    "lemonade", "cocoa", "espresso", "cider", "kefir", "sparkling water", "ginger ale", "chai",
    "horchata", "kombucha", "root beer", "cold brew", "matcha", "hibiscus cooler",
    "elderflower soda", "cream soda", "mango lassi", "ayran", "agua fresca", "cappuccino",
    "cortado", "mocha", "lemon soda", "iced latte", "barley water", "malted milk", "tonic water",
    "vanilla shake", "grape soda", "cherry cola",
];

#[rustfmt::skip]
const OBJECTS: &[&str] = &[
    "lantern", "umbrella", "telescope", "kettle", "compass", "harmonica", "jigsaw puzzle",
    "sketchbook", "fountain pen", "skateboard", "hammock", "wristwatch", "camera", "globe",
    "chess set", "flashlight", "candle", "blanket", "mug", "wallet", "tripod", "microscope",
    "tambourine", "snow globe", "picture frame", "toolbox", "thermos", "cookbook", "houseplant",
    "bird feeder", "sleeping bag", "yoga mat", "pair of binoculars", "magnifying glass",
    "music box", "pocket knife", "water pistol", "paint set", "robot kit", "stopwatch",
    "calculator", "beanie", "pair of mittens", "puzzle box", "hourglass", "easel", "lamp", "rug",
    "vase", "clock", "kaleidoscope", "terrarium", "pinball game", "record player", "headlamp",
    "pencil case", "jewelry box", "rain poncho", "phone stand", "desk fan",
];

#[rustfmt::skip]
const JOBS: &[&str] = &[
    "architect", "pharmacist", "carpenter", "paramedic", "librarian", "welder", "pilot", "florist",
    "plumber", "electrician", "dentist", "nurse", "accountant", "journalist", "translator",
    "beekeeper", "mechanic", "veterinarian", "surveyor", "cartographer", "locksmith", "baker",
    "tailor", "potter", "sculptor", "geologist", "botanist", "chemist", "firefighter", "zookeeper",
    "lifeguard", "barista", "park ranger", "glassblower", "jeweler", "upholsterer", "optician",
    "physiotherapist", "radiologist", "lawyer", "economist", "actuary", "statistician",
    "game designer", "animator", "sound engineer", "stage manager", "choreographer", "midwife",
    "social worker", "auctioneer", "blacksmith", "cobbler", "sommelier", "astronomer",
    "marine biologist", "archivist", "curator", "data analyst", "court reporter",
];

#[rustfmt::skip]
const PLACES: &[&str] = &[
    "Lisbon", "Oslo", "Nairobi", "Kyoto", "Montreal", "Lima", "Reykjavik", "Hanoi", "Tbilisi",
    "Marrakesh", "Edinburgh", "Valparaiso", "Cusco", "Dubrovnik", "Krakow", "Seville", "Porto",
    "Bergen", "Tallinn", "Riga", "Vilnius", "Ljubljana", "Zagreb", "Sofia", "Bucharest",
    "Budapest", "Vienna", "Prague", "Salzburg", "Munich", "Hamburg", "Antwerp", "Ghent", "Bruges",
    "Lyon", "Marseille", "Bordeaux", "Naples", "Florence", "Bologna", "Palermo", "Athens",
    "Thessaloniki", "Izmir", "Beirut", "Amman", "Muscat", "Doha", "Mumbai", "Jaipur", "Kathmandu",
    "Colombo", "Dhaka", "Chiang Mai", "Penang", "Manila", "Cebu", "Jakarta", "Perth", "Hobart",
    "Auckland", "Wellington", "Quebec", "Halifax", "Vancouver", "Seattle", "Denver", "Austin",
    "Memphis", "Savannah", "Boston", "Chicago", "Toronto", "Havana", "Bogota", "Quito",
    "Montevideo", "Asuncion", "La Paz", "Accra", "Dakar", "Lagos", "Kigali", "Addis Ababa",
    "Zanzibar", "Cape Town", "Windhoek", "Casablanca", "Tunis", "Cairo", "Seoul", "Busan",
    "Taipei", "Osaka", "Sapporo", "Ulaanbaatar", "Almaty", "Tashkent", "Baku", "Yerevan",
];

const TOWN_PREFIXES: &[&str] = &[
    "Ash", "Bram", "Cold", "Elder", "Fern", "Glen", "Kings", "Lark", "Mill", "North", "Oak",
    "Pine", "Raven", "Stone", "Thorn", "Willow", "Briar", "Dun", "Hart", "Marl", "Pen", "Sel",
    "Tarn", "Wick", "Holm", "Lang", "Cray", "Bel", "Gal", "Ross",
];
const TOWN_SUFFIXES: &[&str] = &[
    "ford", "brook", "field", "wick", "haven", "mere", "dale", "stead", "ton", "bury", "combe",
    "holt", "ridge", "worth", "moor",
];
const STREET_WORDS: &[&str] = &[
    "Hawthorn", "Juniper", "Linden", "Alder", "Larch", "Sycamore", "Magnolia", "Heron", "Kestrel",
    "Plover", "Osprey", "Curlew", "Mulberry", "Chestnut", "Quarry", "Tanner", "Cooper", "Wheeler",
    "Fletcher", "Orchard", "Meadow", "Harbor", "Lantern", "Granary", "Foundry", "Weaver", "Cobble",
    "Saddler", "Thistle", "Bracken", "Sorrel", "Yarrow", "Elm", "Poplar", "Hazel", "Laurel",
    "Cedar", "Birch", "Aspen", "Rowan",
];
const STREET_TYPES: &[&str] = &[
    "Street", "Lane", "Road", "Avenue", "Close", "Terrace", "Crescent", "Drive",
];

#[rustfmt::skip]
const INSTRUMENTS: &[&str] = &[
    "cello", "oboe", "bassoon", "clarinet", "trombone", "banjo", "mandolin", "harp", "accordion",
    "bagpipes", "viola", "flute", "xylophone", "marimba", "tuba", "sitar", "ukulele", "saxophone",
    "double bass", "harpsichord", "bongos", "steel drum", "theremin", "trumpet", "violin",
    "french horn", "glockenspiel", "lute", "dulcimer", "euphonium",
];

#[rustfmt::skip]
const SPOTS: &[&str] = &[
    "in the hallway closet", "in the garden shed", "under the stairs", "in the attic",
    "in the cellar", "behind the radiator", "in the sewing tin", "on the windowsill",
    "in the toolbox", "inside the piano bench", "in the mailbox", "on top of the fridge",
    "in the linen cupboard", "in the medicine cabinet", "on the mantelpiece", "in the shoebox",
    "in the glove box", "under the doormat", "in the umbrella stand", "in the garage",
    "behind the bookcase", "in the laundry room", "in the spice rack", "in the filing cabinet",
    "under the sink", "in the wardrobe", "in the junk tray", "in the craft cupboard",
    "inside the vase", "in the tool chest", "in the camping crate", "in the guest room",
    "in the pantry", "in the bathroom cabinet", "behind the sofa cushions", "in the hall cupboard",
    "inside the old suitcase", "in the recycling bin", "on the porch step", "in the sideboard",
];

#[rustfmt::skip]
const COLORS: &[&str] = &[
    "teal", "maroon", "crimson", "lavender", "beige", "turquoise", "magenta", "olive", "navy",
    "amber", "coral", "ivory", "indigo", "scarlet", "mint", "peach", "mustard", "burgundy",
    "cobalt", "emerald", "ochre", "rust", "sage", "charcoal", "mauve", "lilac", "tan", "cyan",
    "khaki", "cream", "copper", "bronze", "periwinkle", "aquamarine", "vermilion", "chartreuse",
    "taupe", "fuchsia", "sapphire", "jade",
];

#[rustfmt::skip]
const PLANTS: &[&str] = &[
    "tulips", "basil", "ferns", "peonies", "daffodils", "marigolds", "rosemary", "thyme",
    "zinnias", "dahlias", "hostas", "lilies", "irises", "snapdragons", "foxgloves", "begonias",
    "geraniums", "chives", "strawberries", "pansies", "petunias", "orchids", "succulents",
    "bamboo", "hydrangeas", "lupins", "poppies", "asters", "oregano", "parsley", "dill",
    "cilantro", "lettuce", "radishes", "kale", "squash", "pumpkins", "heather", "crocuses",
    "camellias",
];

#[rustfmt::skip]
const MONTHS: &[&str] = &[
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

#[rustfmt::skip]
const PEOPLE_KEYS: &[&str] = &[
    "Grandpa", "Grandma", "Mom", "Dad", "my sister", "my brother", "my cousin", "my neighbor",
    "my roommate", "my boss", "my best friend", "my nephew", "my niece", "my godmother",
    "my stepdad", "my husband", "my wife", "my partner", "my son", "my daughter", "my grandson",
    "my granddaughter", "our babysitter", "my coworker", "my mentor", "my landlord",
    "my mother-in-law", "my stepsister", "my godfather", "my flatmate",
];

#[rustfmt::skip]
const ROLE_KEYS: &[&str] = &[
    "our new landlord", "the plumber", "my boss", "the coach", "my dentist", "the babysitter",
    "our neighbor", "the intern", "my piano teacher", "the tour guide", "the vet", "the mechanic",
    "my lab partner", "the head chef", "the bus driver", "my pen pal", "the lifeguard",
    "my driving instructor", "the pharmacist", "the night nurse", "the wedding planner",
    "the electrician", "the photographer", "the librarian", "the swim coach", "the dog walker",
    "the gardener", "my study buddy",
];

#[rustfmt::skip]
const PET_KEYS: &[&str] = &[
    "the turtle", "the goldfish", "the hamster", "the parrot", "the kitten", "the puppy",
    "the lizard", "the ferret", "the guinea pig", "the canary", "the gecko", "the pony",
    "the tortoise", "the cockatiel", "the iguana", "the snake", "the chinchilla", "the hedgehog",
    "the duckling", "the budgie", "the axolotl", "the goat", "the lamb", "the piglet",
    "the donkey", "the alpaca", "the hen", "the gerbil", "the cockatoo", "the newt",
];

#[rustfmt::skip]
const ITEM_KEYS: &[&str] = &[
    "the spare key", "my passport", "the phone charger", "the tape measure", "the flashlight",
    "the remote", "my wallet", "the stapler", "the screwdriver", "the extra lightbulb",
    "the parcel", "the receipt", "the library card", "the hammer", "the measuring jug",
    "the first aid kit", "the gift card", "the spare battery", "my sketchbook", "the padlock",
    "the hair dryer", "the bike pump", "the thermometer", "my headphones", "the glue gun",
];

#[rustfmt::skip]
const PAINTABLE_KEYS: &[&str] = &[
    "the fence", "the porch", "the garage door", "the front door", "the canoe", "the shed",
    "the mailbox", "the bathroom", "the hallway", "the bedroom", "the bookcase", "the dresser",
    "the garden bench", "the treehouse", "the bike", "the boat", "the van", "the gate",
    "the playroom", "the kitchen", "the shutters", "the bird house", "the attic", "the study",
];

#[rustfmt::skip]
const APPOINTMENT_KEYS: &[&str] = &[
    "my dentist appointment", "the yoga class", "the team meeting", "my haircut",
    "the pottery class", "my eye exam", "the vet appointment", "choir practice",
    "the climbing session", "my job interview", "the parent meeting", "the plumber visit",
    "the conference call", "the piano recital", "the chess club", "the swim lesson",
    "my physio session", "the carpool pickup", "the movie", "the bake sale", "the fire drill",
    "the flu shot", "the guitar lesson", "the budget review", "the volunteer shift",
];

#[rustfmt::skip]
const EVENT_KEYS: &[&str] = &[
    "the wedding", "the school play", "the family reunion", "the garage sale", "the science fair",
    "the marathon", "the housewarming", "the recital", "the graduation", "the camping trip",
    "the art show", "the open house", "the charity walk", "the street festival",
    "the retirement party", "the anniversary trip", "the move", "the surgery", "the job fair",
    "the cousins' visit", "the quiz night", "the regatta",
];

#[rustfmt::skip]
const BED_KEYS: &[&str] = &[
    "the front bed", "the window box", "the back corner", "the raised bed", "the big planter",
    "the herb patch", "the strip by the fence", "the shady corner", "the rockery",
    "the side border", "the greenhouse", "the wheelbarrow", "the balcony box",
    "the courtyard planter", "the trough", "the allotment",
];

// ---------------------------------------------------------------------------
// Frames: a relation between a key class and a value class, with templates.
//
// Placeholders: {k} {K} user-voice key (capitalised), {kp} {Kp} its possessive;
// {ka} {KA} assistant-voice key, {kap} {KAP} its possessive; {v} {V} value;
// {av} {AV} value with its article; {old} {aold} superseded value. Pair
// templates number the two keys and values 1 and 2.

struct Frame {
    id: &'static str,
    keys: Kc,
    values: Vc,
    art: Art,
    /// "You said {av}." reads naturally for this value class.
    said: bool,
    pair: &'static [&'static str],
    single: &'static [&'static str],
    update: &'static [&'static str],
    ask: &'static [&'static str],
    answer: &'static [&'static str],
    reverse_ask: &'static [&'static str],
    reverse_answer: &'static [&'static str],
    suggest_request: &'static [&'static str],
    suggest_reply: &'static [&'static str],
}

const FRAMES: &[Frame] = &[
    Frame {
        id: "pet_names",
        keys: Kc::Pets,
        values: Vc::PetName,
        art: Art::None,
        said: true,
        pair: &[
            "We have a couple of new pets: {k1} is called {v1} and {k2} is called {v2}.",
            "Our new pets finally have names. {K1} is {v1}, and {k2} is {v2}.",
            "We brought home {k1} and {k2} this week, and we named them {v1} and {v2}, in that order.",
            "The kids named {k1} {v1} and {k2} {v2}.",
        ],
        single: &["{K} is called {v}.", "We named {k} {v}.", "Oh, and {k} is named {v}."],
        update: &[
            "Actually, we renamed {k}. It's {v} now, not {old}.",
            "Small change: {k} is called {v} now instead of {old}.",
            "We changed our minds about {k} and went with {v}.",
        ],
        ask: &[
            "What's {k} called?",
            "What name did we give {k}?",
            "What did we name {k}?",
            "Do you remember what {k} is called?",
        ],
        answer: &["{KA} is called {v}.", "{KA} is named {v}.", "You named {ka} {v}."],
        reverse_ask: &["Which pet is called {v}?", "Which of our pets is {v}?"],
        reverse_answer: &["{KA} is called {v}.", "{V} is {ka}."],
        suggest_request: &[
            "Can you help me name {k1} and {k2}?",
            "We need names for {k1} and {k2}. Any ideas?",
        ],
        suggest_reply: &[
            "How about {v1} for {ka1} and {v2} for {ka2}?",
            "You could call {ka1} {v1} and {ka2} {v2}.",
        ],
    },
    Frame {
        id: "purchases",
        keys: Kc::People,
        values: Vc::Food,
        art: Art::None,
        said: true,
        pair: &[
            "At the market, {k1} bought {v1} and {k2} bought {v2}.",
            "We went shopping today. {K1} picked up {v1}, and {k2} got {v2}.",
            "{K1} came home with {v1}, while {k2} bought {v2}.",
        ],
        single: &["{K} bought {v}.", "{K} picked up {v} at the store."],
        update: &[
            "Wait, I got that wrong. {K} actually bought {v}, not {old}.",
            "Correction: {k} bought {v}, not {old}.",
        ],
        ask: &[
            "What did {k} buy?",
            "What did {k} pick up at the store?",
            "What did {k} come home with?",
        ],
        answer: &["{KA} bought {v}.", "{KA} picked up {v}."],
        reverse_ask: &["Who bought {v}?", "Who picked up {v}?"],
        reverse_answer: &["{KA} bought {v}.", "{KA} did."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "favorite_dishes",
        keys: Kc::People,
        values: Vc::Dish,
        art: Art::None,
        said: true,
        pair: &[
            "{Kp1} favorite dish is {v1}, but {kp2} favorite is {v2}.",
            "{K1} loves {v1}, and {k2} would eat {v2} every night if they could.",
            "When we eat out, {k1} always wants {v1} and {k2} always wants {v2}.",
        ],
        single: &["{Kp} favorite dish is {v}.", "{K} really loves {v}."],
        update: &[
            "Actually, {kp} new favorite is {v}. {K} is tired of {old}.",
            "I was wrong before. {Kp} favorite dish is {v}, not {old}.",
        ],
        ask: &[
            "What's {kp} favorite dish?",
            "What does {k} love to eat?",
            "Which dish does {k} like best?",
        ],
        answer: &["{KAP} favorite dish is {v}.", "{KA} loves {v}."],
        reverse_ask: &["Who loves {v}?", "Whose favorite dish is {v}?"],
        reverse_answer: &["{KA} loves {v}.", "That's {kap} favorite."],
        suggest_request: &[
            "What should I cook for {k1} and {k2}?",
            "I'm cooking for {k1} and {k2}. What should I make?",
        ],
        suggest_reply: &[
            "Maybe {v1} for {ka1} and {v2} for {ka2}.",
            "You could make {v1} for {ka1} and {v2} for {ka2}.",
        ],
    },
    Frame {
        id: "drink_orders",
        keys: Kc::People,
        values: Vc::Drink,
        art: Art::None,
        said: true,
        pair: &[
            "At the café, {k1} ordered {v1} and {k2} had {v2}.",
            "We stopped for drinks. {K1} got {v1}, and {k2} ordered {v2}.",
            "{K1} asked for {v1}, and {k2} went with {v2}.",
        ],
        single: &["{K} ordered {v}.", "{K} had {v} at the café."],
        update: &["Oops, I mixed that up. {K} ordered {v}, not {old}."],
        ask: &[
            "What did {k} order?",
            "What did {k} get to drink?",
            "Which drink did {k} have?",
        ],
        answer: &["{KA} ordered {v}.", "{KA} had {v}."],
        reverse_ask: &["Who ordered {v}?", "Who had {v}?"],
        reverse_answer: &["{KA} ordered {v}.", "{KA} did."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "gifts",
        keys: Kc::People,
        values: Vc::Object,
        art: Art::Indef,
        said: true,
        pair: &[
            "For the holidays I got {k1} {av1} and {k2} {av2}.",
            "I finished my gift shopping: {av1} for {k1} and {av2} for {k2}.",
            "I wrapped the presents today. {K1} is getting {av1}, and {k2} is getting {av2}.",
        ],
        single: &["I got {k} {av}.", "{K} is getting {av} from me."],
        update: &[
            "Change of plans: I'm giving {k} {av} instead of {aold}.",
            "I exchanged {kp} gift. {K} is getting {av} now, not {aold}.",
        ],
        ask: &[
            "What did I get {k}?",
            "What gift is {k} getting?",
            "What am I giving {k}?",
        ],
        answer: &["You got {ka} {av}.", "{KA} is getting {av}."],
        reverse_ask: &["Who is getting {av}?", "Who did I get {av} for?"],
        reverse_answer: &["{KA} is getting {av}.", "That one is for {ka}."],
        suggest_request: &[
            "What should I get {k1} and {k2}?",
            "Any gift ideas for {k1} and {k2}?",
        ],
        suggest_reply: &[
            "Maybe {av1} for {ka1} and {av2} for {ka2}.",
            "How about {av1} for {ka1} and {av2} for {ka2}?",
        ],
    },
    Frame {
        id: "ages",
        keys: Kc::People,
        values: Vc::Age,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} just turned {v1}, and {k2} is {v2}.",
            "{K1} is {v1} years old and {k2} is {v2}.",
            "We had birthdays this month: {k1} turned {v1} and {k2} turned {v2}.",
        ],
        single: &["{K} is {v} years old.", "{K} just turned {v}."],
        update: &["Sorry, I said that wrong. {K} is {v}, not {old}."],
        ask: &[
            "How old is {k}?",
            "What age did {k} just turn?",
            "How old did I say {k} is?",
        ],
        answer: &["{KA} is {v}.", "{KA} is {v} years old."],
        reverse_ask: &["Who is {v} years old?", "Who just turned {v}?"],
        reverse_answer: &["{KA} is {v}.", "{KA} is {v} years old."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "jobs",
        keys: Kc::People,
        values: Vc::Job,
        art: Art::Indef,
        said: false,
        pair: &[
            "{K1} works as {av1}, and {k2} is {av2}.",
            "{K1} just started a job as {av1}. {K2} has been {av2} for years.",
            "In our family, {k1} is {av1} and {k2} is {av2}.",
        ],
        single: &["{K} works as {av}.", "{K} is {av}."],
        update: &["Actually, {k} switched careers and is {av} now, not {aold}."],
        ask: &[
            "What does {k} do for work?",
            "What is {kp} job?",
            "What does {k} do for a living?",
        ],
        answer: &["{KA} is {av}.", "{KA} works as {av}."],
        reverse_ask: &["Who works as {av}?", "Which of them is {av}?"],
        reverse_answer: &["{KA} is {av}.", "{KA} works as {av}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "travel",
        keys: Kc::People,
        values: Vc::Place,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is flying to {v1} next month, and {k2} is heading to {v2}.",
            "Big travel plans this year: {k1} is going to {v1} and {k2} is off to {v2}.",
            "{K1} booked a trip to {v1}, while {k2} chose {v2}.",
        ],
        single: &["{K} is going to {v} next month.", "{K} booked a trip to {v}."],
        update: &["{Kp} plans changed. {K} is going to {v} now instead of {old}."],
        ask: &[
            "Where is {k} going?",
            "Where is {k} traveling to?",
            "Which place is {k} visiting?",
        ],
        answer: &["{KA} is going to {v}.", "{KA} is traveling to {v}."],
        reverse_ask: &["Who is going to {v}?", "Who booked the trip to {v}?"],
        reverse_answer: &["{KA} is going to {v}.", "{KA} is."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "addresses",
        keys: Kc::People,
        values: Vc::Street,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} lives on {v1}, and {k2} lives on {v2}.",
            "{K1} just moved to {v1}. {K2} is still on {v2}.",
            "If you need to visit, {k1} is on {v1} and {k2} is on {v2}.",
        ],
        single: &["{K} lives on {v}.", "{K} moved to {v}."],
        update: &["{K} moved last week. The new place is on {v}, not {old}."],
        ask: &[
            "What's {kp} address?",
            "Where does {k} live?",
            "Where is {kp} place?",
        ],
        answer: &["{KA} lives on {v}.", "{KA} is on {v}."],
        reverse_ask: &["Who lives on {v}?"],
        reverse_answer: &["{KA} lives on {v}.", "{KA} does."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "instruments",
        keys: Kc::People,
        values: Vc::Instrument,
        art: Art::The,
        said: true,
        pair: &[
            "{K1} plays the {v1}, and {k2} plays the {v2}.",
            "In the band, {k1} is on the {v1} and {k2} is on the {v2}.",
            "{K1} practices the {v1} every evening, while {k2} practices the {v2}.",
        ],
        single: &["{K} plays the {v}."],
        update: &["I got that mixed up. {K} plays the {v}, not the {old}."],
        ask: &[
            "Which instrument does {k} play?",
            "What instrument does {k} play?",
            "What does {k} practice every evening?",
        ],
        answer: &["{KA} plays the {v}."],
        reverse_ask: &["Who plays the {v}?"],
        reverse_answer: &["{KA} plays the {v}.", "{KA} does."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "role_names",
        keys: Kc::Roles,
        values: Vc::PersonName,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is called {v1}, and {k2} is called {v2}.",
            "We finally met everyone. {Kp1} name is {v1}, and {kp2} name is {v2}.",
            "Quick update: {k1} is {v1} and {k2} is {v2}.",
        ],
        single: &["{Kp} name is {v}.", "{K} is called {v}."],
        update: &["I misheard before. {Kp} name is {v}, not {old}."],
        ask: &[
            "What's {kp} name?",
            "What is {k} called?",
            "Do you remember {kp} name?",
        ],
        answer: &["{KAP} name is {v}.", "{KA} is called {v}."],
        reverse_ask: &["Who is {v}?", "Which one is {v}?"],
        reverse_answer: &["{V} is {ka}.", "{KA} is called {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "storage",
        keys: Kc::Items,
        values: Vc::Spot,
        art: Art::None,
        said: false,
        pair: &[
            "I put {k1} {v1} and {k2} {v2}.",
            "Before I forget: {k1} is {v1}, and {k2} is {v2}.",
            "I tidied up. {K1} is now {v1}, and {k2} is {v2}.",
        ],
        single: &["I put {k} {v}.", "{K} is {v}."],
        update: &["Actually, I moved {k}. It's {v} now, not {old}."],
        ask: &[
            "Where did I put {k}?",
            "Where is {k}?",
            "Where did I leave {k}?",
        ],
        answer: &["You put {ka} {v}.", "{KA} is {v}."],
        reverse_ask: &["What did I put {v}?", "What's {v}?"],
        reverse_answer: &["You put {ka} {v}.", "{KA} is {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "paint_colors",
        keys: Kc::Paintables,
        values: Vc::Color,
        art: Art::None,
        said: true,
        pair: &[
            "We painted {k1} {v1} and {k2} {v2}.",
            "The painters finished. {K1} is {v1} now, and {k2} is {v2}.",
            "We picked colors at last: {v1} for {k1} and {v2} for {k2}.",
        ],
        single: &["We painted {k} {v}.", "{K} is {v} now."],
        update: &["We repainted {k}. It's {v} now, not {old}."],
        ask: &[
            "What color did we paint {k}?",
            "What color is {k} now?",
            "Which color did we pick for {k}?",
        ],
        answer: &["You painted {ka} {v}.", "{KA} is {v}."],
        reverse_ask: &["What did we paint {v}?", "Which one is {v}?"],
        reverse_answer: &["You painted {ka} {v}.", "{KA} is {v}."],
        suggest_request: &["What colors should we paint {k1} and {k2}?"],
        suggest_reply: &[
            "Try {v1} for {ka1} and {v2} for {ka2}.",
            "I'd go with {v1} for {ka1} and {v2} for {ka2}.",
        ],
    },
    Frame {
        id: "appointment_times",
        keys: Kc::Appointments,
        values: Vc::Time,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is at {v1}, and {k2} is at {v2}.",
            "My calendar is busy: {k1} at {v1} and {k2} at {v2}.",
            "Don't let me forget: {k1} starts at {v1}, and {k2} starts at {v2}.",
        ],
        single: &["{K} is at {v}.", "{K} starts at {v}."],
        update: &["{K} got moved. It's at {v} now, not {old}."],
        ask: &[
            "What time is {k}?",
            "When does {k} start?",
            "What time did I say {k} is?",
        ],
        answer: &["{KA} is at {v}.", "{KA} starts at {v}."],
        reverse_ask: &["What's happening at {v}?", "What do I have at {v}?"],
        reverse_answer: &["{KA} is at {v}.", "You have {ka} at {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "event_dates",
        keys: Kc::Events,
        values: Vc::Date,
        art: Art::None,
        said: true,
        pair: &[
            "{K1} is on {v1}, and {k2} is on {v2}.",
            "We set the dates: {k1} on {v1} and {k2} on {v2}.",
            "Mark your calendar: {k1} is {v1} and {k2} is {v2}.",
        ],
        single: &["{K} is on {v}."],
        update: &["{K} was rescheduled to {v}. It's no longer {old}."],
        ask: &["When is {k}?", "What date is {k}?", "Which date did we set for {k}?"],
        answer: &["{KA} is on {v}."],
        reverse_ask: &["What's happening on {v}?"],
        reverse_answer: &["{KA} is on {v}."],
        suggest_request: &[],
        suggest_reply: &[],
    },
    Frame {
        id: "garden_beds",
        keys: Kc::Beds,
        values: Vc::Plant,
        art: Art::None,
        said: true,
        pair: &[
            "We planted {v1} in {k1} and {v2} in {k2}.",
            "The garden is coming along: {v1} in {k1} and {v2} in {k2}.",
            "This spring {k1} got {v1}, and {k2} got {v2}.",
        ],
        single: &["We planted {v} in {k}."],
        update: &["We dug up {k} and planted {v} there instead of {old}."],
        ask: &[
            "What did we plant in {k}?",
            "What's growing in {k}?",
            "What went into {k}?",
        ],
        answer: &["You planted {v} in {ka}.", "{KA} has {v}."],
        reverse_ask: &["Where did we plant {v}?", "Which spot has {v}?"],
        reverse_answer: &["You planted {v} in {ka}.", "{KA} has {v}."],
        suggest_request: &["What should we plant in {k1} and {k2}?"],
        suggest_reply: &["Try {v1} in {ka1} and {v2} in {ka2}."],
    },
];

/// Distractor exchanges: a user turn and its possible replies.
const FILLERS: &[(&str, &[&str])] = &[
    ("It has been raining all morning here.", &["Rainy mornings can be cozy. I hope you can stay dry.", "That sounds gloomy. Maybe it will clear up later."]),
    ("I just got back from a long walk.", &["That sounds refreshing. Did you enjoy the fresh air?", "Nice! Walking is a great way to clear your head."]),
    ("I'm feeling a little tired today.", &["I'm sorry to hear that. Maybe a short rest would help.", "That happens. Try to take it easy if you can."]),
    ("What is a good way to relax after work?", &["A quiet walk, some music, or a warm shower can help you unwind."]),
    ("I think I need a haircut soon.", &["A fresh haircut can feel great. Do you have a style in mind?"]),
    ("What's the difference between weather and climate?", &["Weather is what happens day to day, while climate is the pattern over many years."]),
    ("Do you like music?", &["I enjoy talking about music. What kind do you like?"]),
    ("I'm trying to drink more water.", &["That's a healthy habit. Keeping a bottle nearby can help."]),
    ("The traffic was terrible this morning.", &["That sounds frustrating. I hope the rest of your day goes more smoothly."]),
    ("How do plants make their own sugar?", &["Plants use sunlight, water and air to make sugar in their leaves. This is called photosynthesis."]),
    ("I can't decide what to watch tonight.", &["Maybe pick something light and funny if you want to relax."]),
    ("Thanks for helping me earlier.", &["You're welcome. I'm glad I could help.", "Anytime! I'm happy to help."]),
    ("Why is the sky dark at night?", &["At night our side of the Earth faces away from the sun, so there is no sunlight to light up the sky."]),
    ("I started learning to knit.", &["That's a nice hobby. Start with something simple, like a dishcloth."]),
    ("It's so windy outside today.", &["Windy days can be wild. Hold on to anything light if you go out."]),
    ("I have a headache.", &["I'm sorry. Drinking some water and resting your eyes might help."]),
    ("Tell me something cheerful.", &["Here is something cheerful: somewhere right now, someone is laughing at a joke they just heard."]),
    ("Do you ever get bored?", &["I don't get bored the way people do, but I enjoy a good conversation."]),
    ("How many minutes are in an hour?", &["There are sixty minutes in an hour."]),
    ("What's another word for happy?", &["You could say glad, cheerful or joyful."]),
    ("I'm nervous about my presentation.", &["That's normal. Practicing out loud a few times can really help."]),
    ("We had a power cut last night.", &["That sounds inconvenient. I hope everything is working again."]),
    ("Can you suggest a word game?", &["Try taking turns naming words that start with the last letter of the previous word."]),
    ("What is the capital of Canada?", &["The capital of Canada is Ottawa."]),
    ("My phone battery keeps dying.", &["Try lowering the screen brightness and closing apps you aren't using."]),
    ("Is it better to study in the morning or at night?", &["It depends on the person. Try both and notice when you focus best."]),
    ("What should I do if I can't sleep?", &["Try a calm routine: dim lights, no screens, and slow breathing."]),
    ("The birds outside are so loud this morning.", &["It sounds like they are having a busy morning."]),
    ("I just made a big pot of soup.", &["That sounds warming. Soup is perfect for a chilly day."]),
    ("How do you spell necessary?", &["It is spelled n-e-c-e-s-s-a-r-y."]),
    ("I'm proud of myself for finishing that project.", &["You should be proud. Finishing a project takes effort."]),
    ("It's finally warm enough to open the windows.", &["That's a nice feeling. Enjoy the fresh breeze."]),
    ("Why do some trees drop their leaves?", &["Many trees shed their leaves in the cold season to save water and energy until spring."]),
    ("Can you help me feel less stressed?", &["Let's take a slow breath together. Breathe in, hold it for a moment, and let it out gently."]),
    ("I'm thinking about learning a new language.", &["That's exciting. Which language are you curious about?"]),
    ("What is the opposite of ancient?", &["The opposite of ancient is modern."]),
    ("How far away is the moon?", &["The moon is about 384,000 kilometers from Earth."]),
    ("I'm excited for the weekend.", &["Weekends are great. Do you have anything fun planned?"]),
    ("What's a polite way to say no?", &["You could say, 'Thank you for asking, but I won't be able to this time.'"]),
    ("Do you know any tongue twisters?", &["Try this one: she sells seashells by the seashore."]),
    ("I feel like the week is going by slowly.", &["Some weeks really drag. Hang in there."]),
    ("What makes thunder?", &["Thunder is the sound of air expanding very fast when lightning heats it."]),
    ("I organized my closet today.", &["Nice work. An organized closet makes mornings easier."]),
    ("How do I stay focused?", &["Work in short blocks, put your phone away, and take small breaks."]),
    ("I just learned how to whistle.", &["That's fun! Whistling takes some practice."]),
    ("It's really hot in here.", &["Opening a window or drinking something cool might help."]),
    ("What does curious mean?", &["Curious means wanting to know or learn about something."]),
    ("I watched the sunset from the hill.", &["That sounds peaceful. Sunsets from a hill are special."]),
    ("Let's talk about something else for a second.", &["Sure. What would you like to talk about?"]),
    ("That reminds me of something funny.", &["I'd love to hear it."]),
    ("What's the tallest mountain in the world?", &["Mount Everest is the tallest mountain above sea level."]),
    ("Do penguins live at the North Pole?", &["No, wild penguins live in the Southern Hemisphere, not at the North Pole."]),
    ("I'm making a shopping list.", &["Good idea. A list makes shopping quicker."]),
    ("How are things with you?", &["Things are fine on my end. How about you?"]),
    ("I finally fixed the squeaky hinge.", &["That must be a relief. Squeaky hinges are so annoying."]),
    ("What causes a rainbow?", &["Sunlight bends and splits inside raindrops, which spreads it into bands of color."]),
    ("I'm going to try a new recipe tonight.", &["Have fun with it! Trying new recipes is a good way to learn."]),
    ("My neighbor plays loud music late at night.", &["That sounds frustrating. A friendly chat might help."]),
    ("How long does it take to boil an egg?", &["For a soft egg, a few minutes; for a firm one, a little longer."]),
    ("I need a short break.", &["Good idea. Stretch, breathe and come back when you're ready."]),
    ("Is a tomato a fruit or a vegetable?", &["Botanically it is a fruit, but cooks usually treat it as a vegetable."]),
    ("What's your favorite season?", &["I don't have one, but many people love autumn for its cool air."]),
    ("I almost missed the bus this morning.", &["Glad you made it! Mornings can be hectic."]),
    ("Can you explain what gravity is?", &["Gravity is the pull that objects with mass have on each other. It keeps us on the ground."]),
    ("I'm a bit bored right now.", &["Maybe try a puzzle, a short walk, or a new song."]),
];

#[rustfmt::skip]
const ACKS: &[&str] = &[
    "Got it.", "Good to know.", "Thanks for telling me.", "Okay, I'll keep that in mind.",
    "Noted!", "That sounds like fun.", "Nice!", "Sounds good.", "How exciting!", "Okay, thanks.",
    "That's nice to hear.", "I'll remember that.",
];
const UPDATE_ACKS: &[&str] = &[
    "Okay, thanks for the update.",
    "Got it, I've noted the change.",
    "Thanks for correcting that.",
    "Okay, noted.",
];
const ACCEPTS: &[&str] = &[
    "Perfect, let's go with those.",
    "I like those. We'll use them.",
    "Great ideas, thank you! Let's do that.",
    "Those work for me.",
];
const ACCEPT_ACKS: &[&str] = &[
    "Great, I'm glad you like them.",
    "Happy to help.",
    "Good choice.",
];
const QUESTION_PREFIXES: &[&str] = &[
    "Quick question: ",
    "Remind me, ",
    "Hey, ",
    "Okay, so ",
    "One more thing: ",
    "Sorry, I forgot. ",
];
const SUGGEST_ASKS: &[&str] = &[
    "What did you suggest for {k}?",
    "Remind me what you picked for {k}.",
    "Which one did you suggest for {k}?",
];
const ABSTAIN_ANSWERS: &[&str] = &[
    "You haven't told me that.",
    "You haven't told me that yet.",
    "I don't know. You didn't mention {ka}.",
    "I'm not sure. You never told me about {ka}.",
    "You haven't said anything about {ka} yet.",
    "I don't know that one. You haven't mentioned {ka}.",
];
const NO_FACT_ABSTAIN_ANSWERS: &[&str] = &[
    "You haven't told me that.",
    "I don't know. You haven't told me that.",
    "I'm not sure. You haven't mentioned it.",
];

// ---------------------------------------------------------------------------
// The world: filtered pools and templates.

#[derive(Clone, Debug)]
struct Key {
    user: String,
    asst: String,
    panel_shared: bool,
}

fn key_from(user: &str, panel: &Panel) -> Key {
    let asst = if let Some(rest) = user.strip_prefix("my ") {
        format!("your {rest}")
    } else if let Some(rest) = user.strip_prefix("our ") {
        format!("your {rest}")
    } else {
        match user {
            "Mom" => "your mom".to_owned(),
            "Dad" => "your dad".to_owned(),
            "Grandpa" => "your grandpa".to_owned(),
            "Grandma" => "your grandma".to_owned(),
            other => other.to_owned(),
        }
    };
    Key {
        panel_shared: panel.shares_content(user),
        user: user.to_owned(),
        asst,
    }
}

struct World {
    panel: Panel,
    values: BTreeMap<Vc, Vec<String>>,
    keys: BTreeMap<Kc, Vec<Key>>,
    person_names: Vec<String>,
    fillers: Vec<(String, Vec<String>)>,
    /// Filtered templates, per frame: one list per template field.
    frames: Vec<FrameT>,
    dropped: BTreeMap<String, Vec<String>>,
}

struct FrameT {
    spec: &'static Frame,
    pair: Vec<&'static str>,
    single: Vec<&'static str>,
    update: Vec<&'static str>,
    ask: Vec<&'static str>,
    answer: Vec<&'static str>,
    reverse_ask: Vec<&'static str>,
    reverse_answer: Vec<&'static str>,
    suggest_request: Vec<&'static str>,
    suggest_reply: Vec<&'static str>,
}

fn template_literal(template: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for ch in template.chars() {
        match ch {
            '{' => {
                depth += 1;
                out.push(' ');
            }
            '}' => depth -= 1,
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

impl World {
    fn new(panel: Panel) -> Result<Self> {
        let mut dropped: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut discard = |kind: &str, item: &str| {
            dropped
                .entry(kind.to_owned())
                .or_default()
                .push(item.to_owned());
        };
        // Values: no content stem in common with any panel string or field.
        let mut values: BTreeMap<Vc, Vec<String>> = BTreeMap::new();
        let mut admit = |class: Vc, raw: Vec<String>, values: &mut BTreeMap<Vc, Vec<String>>| {
            let mut kept = Vec::new();
            for item in raw {
                if content_stems(&item).is_empty()
                    || panel.shares_content(&item)
                    || panel.has_strict(&item)
                {
                    discard("value", &item);
                } else if !kept.contains(&item) {
                    kept.push(item);
                }
            }
            values.insert(class, kept);
        };
        let owned = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        admit(Vc::PersonName, owned(PERSON_NAMES), &mut values);
        let mut pets = owned(PET_NAMES);
        pets.extend(owned(PERSON_NAMES));
        admit(Vc::PetName, pets, &mut values);
        admit(Vc::Food, owned(FOODS), &mut values);
        admit(Vc::Dish, owned(DISHES), &mut values);
        admit(Vc::Drink, owned(DRINKS), &mut values);
        admit(Vc::Object, owned(OBJECTS), &mut values);
        admit(
            Vc::Age,
            (13..=95).map(|n| n.to_string()).collect(),
            &mut values,
        );
        admit(Vc::Job, owned(JOBS), &mut values);
        let mut places = owned(PLACES);
        for p in TOWN_PREFIXES {
            for s in TOWN_SUFFIXES {
                places.push(format!("{p}{s}"));
            }
        }
        admit(Vc::Place, places, &mut values);
        let mut streets = Vec::new();
        for w in STREET_WORDS {
            for t in STREET_TYPES {
                streets.push(format!("{w} {t}"));
            }
        }
        admit(Vc::Street, streets, &mut values);
        admit(Vc::Instrument, owned(INSTRUMENTS), &mut values);
        admit(Vc::Spot, owned(SPOTS), &mut values);
        admit(Vc::Color, owned(COLORS), &mut values);
        let mut times = Vec::new();
        for h in 13..=22 {
            for m in [15, 20, 25, 30, 35, 40, 45, 50, 55] {
                times.push(format!("{h}:{m}"));
            }
        }
        admit(Vc::Time, times, &mut values);
        let mut dates = Vec::new();
        for month in MONTHS {
            for day in 13..=31u32 {
                let suffix = match day {
                    21 | 31 => "st",
                    22 => "nd",
                    23 => "rd",
                    _ => "th",
                };
                dates.push(format!("{month} {day}{suffix}"));
            }
        }
        admit(Vc::Date, dates, &mut values);
        admit(Vc::Plant, owned(PLANTS), &mut values);
        for (class, pool) in &values {
            if pool.len() < 8 {
                return Err(format!(
                    "value pool {class:?} has {} values after the panel filter; at least 8 are needed",
                    pool.len()
                ));
            }
        }
        let person_names = values.get(&Vc::PersonName).cloned().unwrap_or_default();

        // Keys: a key holding a recall-row value word is dropped; one sharing
        // other panel words is kept and flagged.
        let mut keys: BTreeMap<Kc, Vec<Key>> = BTreeMap::new();
        for (class, list) in [
            (Kc::Pets, PET_KEYS),
            (Kc::People, PEOPLE_KEYS),
            (Kc::Roles, ROLE_KEYS),
            (Kc::Items, ITEM_KEYS),
            (Kc::Paintables, PAINTABLE_KEYS),
            (Kc::Appointments, APPOINTMENT_KEYS),
            (Kc::Events, EVENT_KEYS),
            (Kc::Beds, BED_KEYS),
        ] {
            let mut kept = Vec::new();
            for user in list {
                if panel.has_strict(user) {
                    discard("key", user);
                } else {
                    kept.push(key_from(user, &panel));
                }
            }
            if kept.iter().filter(|k| !k.panel_shared).count() < 4 {
                return Err(format!(
                    "key class {class:?} has fewer than 4 keys disjoint from the panels"
                ));
            }
            keys.insert(class, kept);
        }

        let mut fillers = Vec::new();
        for (user, replies) in FILLERS {
            if panel.turn_leak(user).is_some() {
                discard("filler", user);
                continue;
            }
            let kept: Vec<String> = replies
                .iter()
                .filter(|r| {
                    let ok = panel.turn_leak(r).is_none();
                    if !ok {
                        discard("filler_reply", r);
                    }
                    ok
                })
                .map(|r| r.to_string())
                .collect();
            if kept.is_empty() {
                discard("filler", user);
            } else {
                fillers.push((user.to_string(), kept));
            }
        }
        if fillers.len() < 20 {
            return Err(format!(
                "only {} distractor exchanges survive the panel filter",
                fillers.len()
            ));
        }

        let mut frames = Vec::new();
        for spec in FRAMES {
            let mut keep = |field: &str, list: &'static [&'static str]| -> Vec<&'static str> {
                list.iter()
                    .copied()
                    .filter(|t| {
                        let literal = template_literal(t);
                        let ok = !panel.has_strict(&literal)
                            && !grams(&literal).iter().any(|g| panel.grams.contains(g));
                        if !ok {
                            discard(&format!("template:{}:{field}", spec.id), t);
                        }
                        ok
                    })
                    .collect()
            };
            let frame = FrameT {
                spec,
                pair: keep("pair", spec.pair),
                single: keep("single", spec.single),
                update: keep("update", spec.update),
                ask: keep("ask", spec.ask),
                answer: keep("answer", spec.answer),
                reverse_ask: keep("reverse_ask", spec.reverse_ask),
                reverse_answer: keep("reverse_answer", spec.reverse_answer),
                suggest_request: keep("suggest_request", spec.suggest_request),
                suggest_reply: keep("suggest_reply", spec.suggest_reply),
            };
            for (field, list) in [
                ("pair", &frame.pair),
                ("single", &frame.single),
                ("update", &frame.update),
                ("ask", &frame.ask),
                ("answer", &frame.answer),
                ("reverse_ask", &frame.reverse_ask),
                ("reverse_answer", &frame.reverse_answer),
            ] {
                if list.is_empty() {
                    return Err(format!("frame {} has no {field} template left", spec.id));
                }
            }
            frames.push(frame);
        }
        Ok(Self {
            panel,
            values,
            keys,
            person_names,
            fillers,
            frames,
            dropped,
        })
    }
}

// ---------------------------------------------------------------------------
// Dialogues.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Category {
    Binding,
    Update,
    Reverse,
    AssistantStated,
    CrossRelation,
    Abstain,
}

impl Category {
    fn name(self) -> &'static str {
        match self {
            Self::Binding => "binding",
            Self::Update => "update",
            Self::Reverse => "reverse",
            Self::AssistantStated => "assistant_stated",
            Self::CrossRelation => "cross_relation",
            Self::Abstain => "abstain",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    User,
    Assistant,
}

#[derive(Clone, Debug)]
struct Turn {
    role: Role,
    text: String,
    /// A distractor exchange's turn.
    filler: bool,
}

#[derive(Clone, Debug)]
struct Fact {
    frame: usize,
    key: Key,
    value: String,
    old: Option<String>,
    by_assistant: bool,
}

#[derive(Clone, Debug)]
struct Question {
    /// Index of the assistant turn that answers it.
    turn: usize,
    category: Category,
    followup: bool,
    /// Detail: `latest`/`unchanged` for updates, `same_relation`,
    /// `other_relation` or `no_facts` for abstentions.
    detail: &'static str,
    /// Words the answer must contain (all content words); `None` for an
    /// abstention.
    expect: Option<String>,
    /// Stated values the answer must not name.
    forbid: Vec<String>,
    /// Other keys the answer must not name.
    forbid_keys: Vec<String>,
}

#[derive(Clone, Debug)]
struct Dialogue {
    category: Category,
    frames: Vec<&'static str>,
    strict_keys: bool,
    turns: Vec<Turn>,
    questions: Vec<Question>,
    facts: Vec<Fact>,
    keys: Vec<Key>,
}

impl Dialogue {
    fn record(&self, id: &str) -> Value {
        json!({
            "id": id,
            "category": self.category.name(),
            "frames": self.frames,
            "strict_keys": self.strict_keys,
            "messages": self.turns.iter().map(|t| json!({
                "role": match t.role { Role::User => "user", Role::Assistant => "assistant" },
                "content": t.text,
            })).collect::<Vec<_>>(),
            "questions": self.questions.iter().map(|q| json!({
                "turn": q.turn,
                "category": q.category.name(),
                "followup": q.followup,
                "detail": q.detail,
                "expect": q.expect,
                "forbid": q.forbid,
                "forbid_keys": q.forbid_keys,
            })).collect::<Vec<_>>(),
        })
    }
}

fn chance(rng: &mut Rng, p: f64) -> bool {
    ((rng.next_u64() >> 11) as f64) / ((1u64 << 53) as f64) < p
}

fn pick<'a, T>(rng: &mut Rng, items: &'a [T]) -> Result<&'a T> {
    if items.is_empty() {
        return Err("picked from an empty list".into());
    }
    Ok(&items[rng.below(items.len())])
}

fn article(value: &str, art: Art) -> String {
    match art {
        Art::None => value.to_owned(),
        Art::The => format!("the {value}"),
        Art::Indef => {
            let vowel = value
                .chars()
                .next()
                .is_some_and(|c| "aeiouAEIOU".contains(c));
            if vowel {
                format!("an {value}")
            } else {
                format!("a {value}")
            }
        }
    }
}

/// Fill `{name}` placeholders from `slots`; an unknown one is an error.
fn fill(template: &str, slots: &BTreeMap<String, String>) -> Result<String> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after
            .find('}')
            .ok_or_else(|| format!("unclosed placeholder in {template:?}"))?;
        let name = &after[..end];
        let value = slots
            .get(name)
            .ok_or_else(|| format!("no slot {name} for {template:?}"))?;
        out.push_str(value);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn possessive(text: &str) -> String {
    format!("{text}'s")
}

fn key_slots(slots: &mut BTreeMap<String, String>, suffix: &str, key: &Key) {
    let mut put = |name: &str, value: String| {
        slots.insert(format!("{name}{suffix}"), value);
    };
    put("k", key.user.clone());
    put("K", cap(&key.user));
    put("kp", possessive(&key.user));
    put("Kp", cap(&possessive(&key.user)));
    put("ka", key.asst.clone());
    put("KA", cap(&key.asst));
    put("kap", possessive(&key.asst));
    put("KAP", cap(&possessive(&key.asst)));
}

fn value_slots(slots: &mut BTreeMap<String, String>, suffix: &str, value: &str, art: Art) {
    let with = article(value, art);
    slots.insert(format!("v{suffix}"), value.to_owned());
    slots.insert(format!("V{suffix}"), cap(value));
    slots.insert(format!("av{suffix}"), with.clone());
    slots.insert(format!("AV{suffix}"), cap(&with));
}

fn fact_slots(fact: &Fact, art: Art) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    key_slots(&mut slots, "", &fact.key);
    value_slots(&mut slots, "", &fact.value, art);
    if let Some(old) = &fact.old {
        slots.insert("old".into(), old.clone());
        slots.insert("aold".into(), article(old, art));
    }
    slots
}

fn pair_slots(a: &Fact, b: &Fact, art: Art) -> BTreeMap<String, String> {
    let mut slots = BTreeMap::new();
    key_slots(&mut slots, "1", &a.key);
    key_slots(&mut slots, "2", &b.key);
    value_slots(&mut slots, "1", &a.value, art);
    value_slots(&mut slots, "2", &b.value, art);
    slots
}

/// Draw state: stems already used by keys and values in this dialogue.
struct Draw<'w> {
    world: &'w World,
    taken: BTreeSet<String>,
    strict: bool,
}

impl Draw<'_> {
    fn fresh(&self, text: &str) -> bool {
        let stems = content_stems(text);
        !stems.is_empty() && stems.iter().all(|s| !self.taken.contains(s))
    }

    fn take(&mut self, text: &str) {
        self.taken.extend(content_stems(text));
    }

    fn key(&mut self, rng: &mut Rng, class: Kc) -> Result<Option<Key>> {
        let world = self.world;
        let pool = world
            .keys
            .get(&class)
            .ok_or_else(|| format!("no keys for {class:?}"))?;
        for _ in 0..40 {
            let key = if class == Kc::People && chance(rng, 0.5) {
                let name = pick(rng, &world.person_names)?;
                Key {
                    user: name.clone(),
                    asst: name.clone(),
                    panel_shared: false,
                }
            } else {
                pick(rng, pool)?.clone()
            };
            if (self.strict && key.panel_shared) || !self.fresh(&key.user) || !self.fresh(&key.asst)
            {
                continue;
            }
            self.take(&key.user);
            self.take(&key.asst);
            return Ok(Some(key));
        }
        Ok(None)
    }

    fn value(&mut self, rng: &mut Rng, class: Vc) -> Result<Option<String>> {
        let world = self.world;
        for _ in 0..40 {
            let candidate = match class {
                Vc::PetName | Vc::PersonName if chance(rng, 0.5) => synthetic_name(rng)?,
                _ => pick(
                    rng,
                    world
                        .values
                        .get(&class)
                        .ok_or_else(|| format!("no values for {class:?}"))?,
                )?
                .clone(),
            };
            let panel = &world.panel;
            if panel.shares_content(&candidate)
                || panel.has_strict(&candidate)
                || !self.fresh(&candidate)
            {
                continue;
            }
            self.take(&candidate);
            return Ok(Some(candidate));
        }
        Ok(None)
    }
}

fn synthetic_name(rng: &mut Rng) -> Result<String> {
    const ONSETS: &[&str] = &[
        "b", "d", "f", "g", "k", "l", "m", "n", "p", "r", "s", "t", "v", "z", "br", "dr", "kl",
        "st", "tr", "sh", "th", "ch", "j", "w",
    ];
    const VOWELS: &[&str] = &["a", "e", "i", "o", "u", "ai", "ei", "ou", "ia"];
    const CODAS: &[&str] = &["", "", "", "n", "l", "r", "s", "m", "k", "th"];
    let syllables = 2 + usize::from(chance(rng, 0.25));
    let mut name = String::new();
    for i in 0..syllables {
        if i > 0 || chance(rng, 0.8) {
            name.push_str(pick(rng, ONSETS)?);
        }
        name.push_str(pick(rng, VOWELS)?);
        if i + 1 == syllables || chance(rng, 0.3) {
            name.push_str(pick(rng, CODAS)?);
        }
    }
    Ok(cap(&name))
}

/// Outcome of one draw: a dialogue or the reason it was redrawn.
enum Drawn {
    Ok(Dialogue),
    Redraw(&'static str),
}

fn pick_category(rng: &mut Rng) -> Category {
    let total: f64 = PRIMARY_SHARES.iter().map(|(_, s)| s).sum();
    let mut x = ((rng.next_u64() >> 11) as f64) / ((1u64 << 53) as f64) * total;
    for (category, share) in PRIMARY_SHARES {
        if x < share {
            return category;
        }
        x -= share;
    }
    Category::Binding
}

struct Builder<'w> {
    draw: Draw<'w>,
    turns: Vec<Turn>,
    facts: Vec<Fact>,
    keys: Vec<Key>,
    frames: Vec<&'static str>,
    questions: Vec<Question>,
    used_fillers: BTreeSet<usize>,
}

impl<'w> Builder<'w> {
    fn world(&self) -> &'w World {
        self.draw.world
    }

    fn user(&mut self, text: String) {
        self.turns.push(Turn {
            role: Role::User,
            text,
            filler: false,
        });
    }

    fn assistant(&mut self, text: String) {
        self.turns.push(Turn {
            role: Role::Assistant,
            text,
            filler: false,
        });
    }

    fn filler(&mut self, rng: &mut Rng) -> Result<()> {
        let fillers = &self.world().fillers;
        for _ in 0..20 {
            let index = rng.below(fillers.len());
            if self.used_fillers.contains(&index) {
                continue;
            }
            self.used_fillers.insert(index);
            let (user, replies) = &fillers[index];
            let reply = pick(rng, replies)?.clone();
            self.turns.push(Turn {
                role: Role::User,
                text: user.clone(),
                filler: true,
            });
            self.turns.push(Turn {
                role: Role::Assistant,
                text: reply,
                filler: true,
            });
            return Ok(());
        }
        Ok(())
    }

    fn fillers(&mut self, rng: &mut Rng, count: usize) -> Result<()> {
        for _ in 0..count {
            self.filler(rng)?;
        }
        Ok(())
    }

    fn note_frame(&mut self, frame: usize) {
        let id = self.world().frames[frame].spec.id;
        if !self.frames.contains(&id) {
            self.frames.push(id);
        }
    }

    /// Draw `n` facts of `frame` (keys and values stem-disjoint from the rest).
    fn new_facts(&mut self, rng: &mut Rng, frame: usize, n: usize) -> Result<Option<Vec<Fact>>> {
        let spec = self.world().frames[frame].spec;
        let mut facts = Vec::new();
        for _ in 0..n {
            let Some(key) = self.draw.key(rng, spec.keys)? else {
                return Ok(None);
            };
            let Some(value) = self.draw.value(rng, spec.values)? else {
                return Ok(None);
            };
            facts.push(Fact {
                frame,
                key,
                value,
                old: None,
                by_assistant: false,
            });
        }
        Ok(Some(facts))
    }

    fn ack(&mut self, rng: &mut Rng, facts: &[Fact]) -> Result<()> {
        // Sometimes the assistant restates the facts, so values also appear in
        // assistant turns.
        if chance(rng, 0.15) {
            let mut parts = vec!["Got it.".to_owned()];
            for fact in facts {
                let frame = &self.world().frames[fact.frame];
                let template = pick(rng, &frame.answer)?;
                parts.push(fill(template, &fact_slots(fact, frame.spec.art))?);
            }
            self.assistant(parts.join(" "));
        } else {
            self.assistant(pick(rng, ACKS)?.to_string());
        }
        Ok(())
    }

    /// A user turn stating `facts` (one or two of one frame).
    fn state(&mut self, rng: &mut Rng, facts: &[Fact]) -> Result<()> {
        let frame = &self.world().frames[facts[0].frame];
        let text = match facts {
            [a, b] => fill(pick(rng, &frame.pair)?, &pair_slots(a, b, frame.spec.art))?,
            [a] => fill(pick(rng, &frame.single)?, &fact_slots(a, frame.spec.art))?,
            _ => return Err("a statement holds one or two facts".into()),
        };
        self.user(text);
        self.ack(rng, facts)?;
        self.note_frame(facts[0].frame);
        for fact in facts {
            self.keys.push(fact.key.clone());
        }
        self.facts.extend(facts.iter().cloned());
        Ok(())
    }

    fn question_text(&self, rng: &mut Rng, text: String) -> Result<String> {
        let starts_wh = ["What", "Where", "Who", "Which", "When", "How"]
            .iter()
            .any(|w| text.starts_with(w));
        if starts_wh && chance(rng, 0.3) {
            let prefix = pick(rng, QUESTION_PREFIXES)?;
            if prefix.ends_with(". ") {
                Ok(format!("{prefix}{text}"))
            } else {
                Ok(format!("{prefix}{}", lower_first(&text)))
            }
        } else {
            Ok(text)
        }
    }

    /// Ask about fact `index` and answer it.
    fn ask_fact(
        &mut self,
        rng: &mut Rng,
        index: usize,
        category: Category,
        followup: bool,
        detail: &'static str,
    ) -> Result<()> {
        let fact = self.facts[index].clone();
        let frame = &self.world().frames[fact.frame];
        let art = frame.spec.art;
        let slots = fact_slots(&fact, art);
        let reverse = category == Category::Reverse;
        let ask = if reverse {
            pick(rng, &frame.reverse_ask)?
        } else if fact.by_assistant && chance(rng, 0.6) {
            pick(rng, SUGGEST_ASKS)?
        } else {
            pick(rng, &frame.ask)?
        };
        let question = self.question_text(rng, fill(ask, &slots)?)?;
        let answer_template: &str = if reverse {
            if chance(rng, 0.7) {
                *pick(rng, &frame.reverse_answer)?
            } else {
                "{KA}."
            }
        } else {
            let r = rng.below(100);
            if r < 55 {
                *pick(rng, &frame.answer)?
            } else if r < 75 {
                "{AV}."
            } else if fact.old.is_some() {
                *pick(rng, &["You changed it to {av}.", "It's {av} now."])?
            } else if fact.by_assistant {
                "I suggested {av} for {ka}."
            } else if frame.spec.said {
                *pick(rng, &["You said {av}.", "You told me {av}."])?
            } else {
                *pick(rng, &frame.answer)?
            }
        };
        let answer = fill(answer_template, &slots)?;
        let mut forbid: Vec<String> = self
            .facts
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != index)
            .map(|(_, f)| f.value.clone())
            .collect();
        forbid.extend(self.facts.iter().filter_map(|f| f.old.clone()));
        if reverse {
            // The asked value is in the question; the other values stay forbidden.
            forbid.retain(|v| v != &fact.value);
        }
        let forbid_keys: Vec<String> = self
            .keys
            .iter()
            .filter(|k| k.user != fact.key.user)
            .map(|k| k.asst.clone())
            .collect();
        let expect = if reverse {
            fact.key.asst.clone()
        } else {
            fact.value.clone()
        };
        self.user(question);
        self.assistant(answer);
        self.questions.push(Question {
            turn: self.turns.len() - 1,
            category,
            followup,
            detail,
            expect: Some(expect),
            forbid,
            forbid_keys,
        });
        Ok(())
    }

    /// Ask about an unstated key and abstain.
    fn ask_unknown(
        &mut self,
        rng: &mut Rng,
        frame: usize,
        followup: bool,
        detail: &'static str,
    ) -> Result<bool> {
        let spec = self.world().frames[frame].spec;
        let Some(key) = self.draw.key(rng, spec.keys)? else {
            return Ok(false);
        };
        let ftemplates = &self.world().frames[frame];
        let mut slots = BTreeMap::new();
        key_slots(&mut slots, "", &key);
        let asked = fill(pick(rng, &ftemplates.ask)?, &slots)?;
        let question = self.question_text(rng, asked)?;
        let answers = if detail == "no_facts" {
            NO_FACT_ABSTAIN_ANSWERS
        } else {
            ABSTAIN_ANSWERS
        };
        let answer = fill(pick(rng, answers)?, &slots)?;
        let mut forbid: Vec<String> = self.facts.iter().map(|f| f.value.clone()).collect();
        forbid.extend(self.facts.iter().filter_map(|f| f.old.clone()));
        let forbid_keys = self.keys.iter().map(|k| k.asst.clone()).collect::<Vec<_>>();
        self.note_frame(frame);
        self.keys.push(key);
        self.user(question);
        self.assistant(answer);
        self.questions.push(Question {
            turn: self.turns.len() - 1,
            category: Category::Abstain,
            followup,
            detail,
            expect: None,
            forbid,
            forbid_keys,
        });
        Ok(true)
    }

    fn other_frame(&self, rng: &mut Rng, not: &[usize], need_suggest: bool) -> Result<usize> {
        let frames = &self.world().frames;
        let candidates: Vec<usize> = (0..frames.len())
            .filter(|i| !not.contains(i))
            .filter(|i| {
                !need_suggest
                    || (!frames[*i].suggest_request.is_empty()
                        && !frames[*i].suggest_reply.is_empty())
            })
            .collect();
        Ok(*pick(rng, &candidates)?)
    }
}

/// State one or two facts of the secondary relation, if there is one.
fn state_secondary(b: &mut Builder<'_>, rng: &mut Rng, secondary: Option<usize>) -> Result<bool> {
    if let Some(frame) = secondary {
        let n = 1 + usize::from(chance(rng, 0.5));
        let Some(facts) = b.new_facts(rng, frame, n)? else {
            return Ok(false);
        };
        b.state(rng, &facts)?;
    }
    Ok(true)
}

fn draw_dialogue(world: &World, rng: &mut Rng) -> Result<Drawn> {
    let category = pick_category(rng);
    let strict = chance(rng, STRICT_KEY_SHARE);
    let mut b = Builder {
        draw: Draw {
            world,
            taken: BTreeSet::new(),
            strict,
        },
        turns: Vec::new(),
        facts: Vec::new(),
        keys: Vec::new(),
        frames: Vec::new(),
        questions: Vec::new(),
        used_fillers: BTreeSet::new(),
    };
    let distractors = rng.below(MAX_DISTRACTORS + 1);
    let pre = usize::from(distractors > 0 && chance(rng, 0.25));
    let mut remaining = distractors - pre;
    b.fillers(rng, pre)?;

    let primary = b.other_frame(rng, &[], category == Category::AssistantStated)?;
    let no_facts = category == Category::Abstain && chance(rng, 0.25);
    // A second relation's facts, as context the questions must ignore.
    let secondary = if !no_facts && category != Category::CrossRelation && chance(rng, 0.35) {
        Some(b.other_frame(rng, &[primary], false)?)
    } else {
        None
    };
    let secondary_first = chance(rng, 0.5);
    if secondary_first && !state_secondary(&mut b, rng, secondary)? {
        return Ok(Drawn::Redraw("key_or_value_pool"));
    }

    let mut primary_facts: Vec<usize> = Vec::new();
    let mut updated: Option<usize> = None;
    match category {
        Category::CrossRelation => {
            let other = b.other_frame(rng, &[primary], false)?;
            let (Some(a), Some(c)) = (b.new_facts(rng, primary, 1)?, b.new_facts(rng, other, 1)?)
            else {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            };
            let fa = &world.frames[primary];
            let fc = &world.frames[other];
            let text = format!(
                "{} {}",
                fill(pick(rng, &fa.single)?, &fact_slots(&a[0], fa.spec.art))?,
                fill(pick(rng, &fc.single)?, &fact_slots(&c[0], fc.spec.art))?
            );
            b.user(text);
            let both = [a[0].clone(), c[0].clone()];
            b.ack(rng, &both)?;
            b.note_frame(primary);
            b.note_frame(other);
            for fact in both {
                b.keys.push(fact.key.clone());
                primary_facts.push(b.facts.len());
                b.facts.push(fact);
            }
        }
        Category::AssistantStated => {
            let Some(mut facts) = b.new_facts(rng, primary, 2)? else {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            };
            let frame = &world.frames[primary];
            let slots = pair_slots(&facts[0], &facts[1], frame.spec.art);
            b.user(fill(pick(rng, &frame.suggest_request)?, &slots)?);
            b.assistant(fill(pick(rng, &frame.suggest_reply)?, &slots)?);
            b.user(pick(rng, ACCEPTS)?.to_string());
            b.assistant(pick(rng, ACCEPT_ACKS)?.to_string());
            b.note_frame(primary);
            for fact in &mut facts {
                fact.by_assistant = true;
                b.keys.push(fact.key.clone());
                primary_facts.push(b.facts.len());
                b.facts.push(fact.clone());
            }
        }
        _ if no_facts => {}
        _ => {
            // Binding, update, reverse and same-relation abstention: two keys
            // of one relation in one turn; an update sometimes states a single
            // key (the latest value still wins).
            let single = category == Category::Update && chance(rng, 0.4);
            let n = if single { 1 } else { 2 };
            let Some(facts) = b.new_facts(rng, primary, n)? else {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            };
            let start = b.facts.len();
            b.state(rng, &facts)?;
            primary_facts.extend(start..b.facts.len());
            if category == Category::Binding && chance(rng, 0.3) {
                if let Some(third) = b.new_facts(rng, primary, 1)? {
                    let start = b.facts.len();
                    b.state(rng, &third)?;
                    primary_facts.extend(start..b.facts.len());
                }
            }
        }
    }
    if !secondary_first && !state_secondary(&mut b, rng, secondary)? {
        return Ok(Drawn::Redraw("key_or_value_pool"));
    }

    if category == Category::Update {
        let before = rng.below(remaining.min(2) + 1);
        b.fillers(rng, before)?;
        remaining -= before;
        let index = *pick(rng, &primary_facts)?;
        let frame_index = b.facts[index].frame;
        let spec = world.frames[frame_index].spec;
        let Some(new_value) = b.draw.value(rng, spec.values)? else {
            return Ok(Drawn::Redraw("key_or_value_pool"));
        };
        let fact = &mut b.facts[index];
        fact.old = Some(std::mem::replace(&mut fact.value, new_value));
        let slots = fact_slots(&b.facts[index], spec.art);
        let template = pick(rng, &world.frames[frame_index].update)?;
        b.user(fill(template, &slots)?);
        b.assistant(pick(rng, UPDATE_ACKS)?.to_string());
        updated = Some(index);
    }

    let followups = if no_facts {
        0
    } else {
        let r = rng.below(100);
        if r < 65 {
            0
        } else if r < 90 {
            1
        } else {
            2
        }
    };
    let between = if followups > 0 && remaining > 0 && chance(rng, 0.5) {
        1
    } else {
        0
    };
    b.fillers(rng, remaining - between)?;

    // The first question.
    let mut asked: BTreeSet<usize> = BTreeSet::new();
    match category {
        Category::Abstain => {
            let (frame, detail) = if no_facts {
                (b.other_frame(rng, &[], false)?, "no_facts")
            } else if chance(rng, 0.65) {
                (primary, "same_relation")
            } else {
                let mut not = vec![primary];
                if let Some(s) = secondary {
                    not.push(s);
                }
                (b.other_frame(rng, &not, false)?, "other_relation")
            };
            if !b.ask_unknown(rng, frame, false, detail)? {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            }
        }
        Category::Update => {
            let target = updated.ok_or("an update without an updated fact")?;
            let others: Vec<usize> = primary_facts
                .iter()
                .copied()
                .filter(|i| *i != target)
                .collect();
            let (index, detail) = if !others.is_empty() && chance(rng, 0.3) {
                (*pick(rng, &others)?, "unchanged")
            } else {
                (target, "latest")
            };
            b.ask_fact(rng, index, Category::Update, false, detail)?;
            asked.insert(index);
        }
        _ => {
            let index = *pick(rng, &primary_facts)?;
            b.ask_fact(rng, index, category, false, "")?;
            asked.insert(index);
        }
    }

    for f in 0..followups {
        if f == 0 && between > 0 {
            b.fillers(rng, between)?;
        }
        let open: Vec<usize> = (0..b.facts.len()).filter(|i| !asked.contains(i)).collect();
        if open.is_empty() && !chance(rng, 0.3) {
            break;
        }
        if open.is_empty() || chance(rng, FOLLOWUP_ABSTAIN) {
            let frame = b.facts.first().map(|f| f.frame).unwrap_or(primary);
            if !b.ask_unknown(rng, frame, true, "same_relation")? {
                return Ok(Drawn::Redraw("key_or_value_pool"));
            }
        } else {
            let index = *pick(rng, &open)?;
            let category = if Some(index) == updated {
                Category::Update
            } else {
                Category::Binding
            };
            let detail = if category == Category::Update {
                "latest"
            } else {
                ""
            };
            b.ask_fact(rng, index, category, true, detail)?;
            asked.insert(index);
        }
    }

    let dialogue = Dialogue {
        category,
        frames: b.frames,
        strict_keys: strict,
        turns: b.turns,
        questions: b.questions,
        facts: b.facts,
        keys: b.keys,
    };
    // Redraw on any leak or collision; then the semantic checks must hold.
    if let Some(reason) = collision(&dialogue) {
        return Ok(Drawn::Redraw(reason));
    }
    for turn in &dialogue.turns {
        if let Some(reason) = world.panel.turn_leak(&turn.text) {
            return Ok(Drawn::Redraw(reason));
        }
    }
    check_dialogue(&dialogue)?;
    Ok(Drawn::Ok(dialogue))
}

/// A distractor or acknowledgement turn naming a key or value of the
/// dialogue, or an abstention key that appears before its question.
fn collision(d: &Dialogue) -> Option<&'static str> {
    let mut stems = BTreeSet::new();
    for fact in &d.facts {
        stems.extend(content_stems(&fact.value));
        if let Some(old) = &fact.old {
            stems.extend(content_stems(old));
        }
    }
    for key in &d.keys {
        stems.extend(content_stems(&key.user));
        stems.extend(content_stems(&key.asst));
    }
    for turn in d.turns.iter().filter(|t| t.filler) {
        if content_stems(&turn.text).iter().any(|s| stems.contains(s)) {
            return Some("distractor_names_a_key_or_value");
        }
    }
    for q in d.questions.iter().filter(|q| q.expect.is_none()) {
        // The abstention key is the last key pushed for it; it must not appear
        // in any turn before its question.
        let question_turn = q.turn - 1;
        let asked = content_stems(&d.turns[question_turn].text);
        let stated: BTreeSet<String> = d
            .facts
            .iter()
            .flat_map(|f| content_stems(&f.key.user))
            .collect();
        let fresh: Vec<&String> = asked.iter().filter(|s| !stated.contains(*s)).collect();
        for turn in &d.turns[..question_turn] {
            let t = content_stems(&turn.text);
            if fresh.iter().any(|s| t.contains(*s) && !is_generic(s)) {
                return Some("abstention_key_mentioned_earlier");
            }
        }
    }
    None
}

/// Question words that are not part of a key ("called", "name", ...).
fn is_generic(stem: &str) -> bool {
    const GENERIC: &[&str] = &[
        "called",
        "name",
        "named",
        "buy",
        "pick",
        "store",
        "come",
        "home",
        "favorite",
        "dish",
        "love",
        "eat",
        "best",
        "order",
        "drink",
        "gift",
        "getting",
        "giving",
        "old",
        "age",
        "turn",
        "work",
        "job",
        "living",
        "traveling",
        "place",
        "visiting",
        "street",
        "live",
        "instrument",
        "play",
        "practice",
        "evening",
        "put",
        "leave",
        "color",
        "paint",
        "time",
        "start",
        "date",
        "set",
        "plant",
        "growing",
        "went",
        "remember",
        "quick",
        "question",
        "remind",
        "sorry",
        "forgot",
        "thing",
        "give",
        "wait",
        "year",
        "years",
        "spring",
        "new",
    ];
    GENERIC.contains(&stem)
}

/// The semantic contract of every question; an error here is a generator bug.
fn check_dialogue(d: &Dialogue) -> Result<()> {
    if d.turns.is_empty() || d.questions.is_empty() {
        return Err("a dialogue without turns or questions".into());
    }
    for (i, turn) in d.turns.iter().enumerate() {
        let expected = if i % 2 == 0 {
            Role::User
        } else {
            Role::Assistant
        };
        if turn.role != expected {
            return Err(format!("turn {i} breaks user/assistant alternation"));
        }
        if turn.text.contains('\n') || turn.text.contains("<|") {
            return Err(format!("turn {i} holds a newline or special-token text"));
        }
    }
    for q in &d.questions {
        let answer = content_stems(&d.turns[q.turn].text);
        let asked = content_stems(&d.turns[q.turn - 1].text);
        if let Some(expect) = &q.expect {
            let want = content_stems(expect);
            if want.is_empty() || !want.iter().all(|s| answer.contains(s)) {
                return Err(format!(
                    "answer {:?} lacks {expect:?}",
                    d.turns[q.turn].text
                ));
            }
            if q.category != Category::Reverse && want.iter().any(|s| asked.contains(s)) {
                return Err(format!(
                    "question {:?} names its own answer",
                    d.turns[q.turn - 1].text
                ));
            }
        }
        for value in &q.forbid {
            if content_stems(value).iter().any(|s| answer.contains(s)) {
                return Err(format!(
                    "answer {:?} names the distractor {value:?}",
                    d.turns[q.turn].text
                ));
            }
        }
        for key in &q.forbid_keys {
            if content_stems(key).iter().any(|s| answer.contains(s)) {
                return Err(format!(
                    "answer {:?} names the other key {key:?}",
                    d.turns[q.turn].text
                ));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Splits.

#[derive(Default)]
struct Tally {
    dialogues: usize,
    turns: usize,
    questions: BTreeMap<String, usize>,
    question_details: BTreeMap<String, usize>,
    primary: BTreeMap<String, usize>,
    frames: BTreeMap<String, usize>,
    redraws: BTreeMap<String, usize>,
    strict_key_dialogues: usize,
    distractor_exchanges: BTreeMap<usize, usize>,
    questions_per_dialogue: BTreeMap<usize, usize>,
    /// Value stems shared with the panels (must stay empty).
    value_stems_shared: BTreeSet<String>,
    /// Key words shared with the panels, with dialogue counts.
    key_words_shared: BTreeMap<String, usize>,
    dialogues_with_panel_key: usize,
    tokens: usize,
    response_tokens: usize,
    recall_response_tokens: usize,
    max_tokens: usize,
}

impl Tally {
    fn add(&mut self, d: &Dialogue, panel: &Panel) {
        self.dialogues += 1;
        self.turns += d.turns.len();
        *self.primary.entry(d.category.name().into()).or_default() += 1;
        for q in &d.questions {
            *self.questions.entry(q.category.name().into()).or_default() += 1;
            let detail = format!(
                "{}{}{}",
                q.category.name(),
                if q.detail.is_empty() { "" } else { ":" },
                q.detail
            );
            *self.question_details.entry(detail).or_default() += 1;
            if q.followup {
                *self.question_details.entry("followup".into()).or_default() += 1;
            }
        }
        for f in &d.frames {
            *self.frames.entry((*f).into()).or_default() += 1;
        }
        self.strict_key_dialogues += usize::from(d.strict_keys);
        let fillers = d.turns.iter().filter(|t| t.filler).count() / 2;
        *self.distractor_exchanges.entry(fillers).or_default() += 1;
        *self
            .questions_per_dialogue
            .entry(d.questions.len())
            .or_default() += 1;
        for fact in &d.facts {
            for v in std::iter::once(&fact.value).chain(fact.old.iter()) {
                for s in content_stems(v) {
                    if panel.content.contains(&s) {
                        self.value_stems_shared.insert(s);
                    }
                }
            }
        }
        let mut shared_here = false;
        for key in &d.keys {
            for s in content_stems(&key.user) {
                if panel.content.contains(&s) {
                    *self.key_words_shared.entry(s).or_default() += 1;
                    shared_here = true;
                }
            }
        }
        self.dialogues_with_panel_key += usize::from(shared_here);
    }

    fn json(&self) -> Value {
        let total_q: usize = self.questions.values().sum();
        json!({
            "dialogues": self.dialogues,
            "turns": self.turns,
            "questions": total_q,
            "questions_per_category": self.questions,
            "question_share_per_category": self.questions.iter()
                .map(|(k, n)| (k.clone(), *n as f64 / total_q.max(1) as f64))
                .collect::<BTreeMap<_, _>>(),
            "questions_per_detail": self.question_details,
            "dialogues_per_primary_category": self.primary,
            "dialogues_per_relation": self.frames,
            "distractor_exchanges_histogram": self.distractor_exchanges,
            "questions_per_dialogue_histogram": self.questions_per_dialogue,
            "strict_key_dialogues": self.strict_key_dialogues,
            "redraws": self.redraws,
            "tokens": self.tokens,
            "response_tokens": self.response_tokens,
            "recall_answer_tokens": self.recall_response_tokens,
            "tokens_per_dialogue": self.tokens as f64 / self.dialogues.max(1) as f64,
            "max_dialogue_tokens": self.max_tokens,
        })
    }
}

struct SplitSpec<'a> {
    name: &'static str,
    label: &'static str,
    seed: u64,
    dialogues: usize,
    token_budget: usize,
    samples: usize,
    dir: &'a Path,
}

struct Encoding<'t> {
    tokenizer: &'t ByteBpeTokenizer,
    protocol: DialogueProtocol,
    version: u8,
    context: usize,
    vocab: u32,
}

/// Generate dialogues until the count or budget; stream tokens and mask; write
/// the samples file and the split's manifest. Returns the tally.
fn write_split(world: &World, enc: &Encoding, spec: &SplitSpec) -> Result<Tally> {
    fs::create_dir_all(spec.dir).map_err(|e| e.to_string())?;
    let encoder: DialogueEncoder<'_> = enc
        .protocol
        .bind(enc.tokenizer)
        .map_err(|e| e.to_string())?;
    let tokens_path = spec.dir.join("tokens.u16");
    let mask_path = spec.dir.join("response_mask.u8");
    let samples_name = if spec.name == "dev" {
        "dialogues.jsonl"
    } else {
        "samples.jsonl"
    };
    let samples_path = spec.dir.join(samples_name);
    let mut writer = CorpusWriter::create(&tokens_path, enc.vocab).map_err(|e| e.to_string())?;
    let mut mask_out = BufWriter::new(fs::File::create(&mask_path).map_err(|e| e.to_string())?);
    let mut samples_out =
        BufWriter::new(fs::File::create(&samples_path).map_err(|e| e.to_string())?);
    let mut rng = Rng::new(spec.seed);
    let mut tally = Tally::default();
    let mut redraws_total = 0usize;
    loop {
        if spec.dialogues > 0 && tally.dialogues >= spec.dialogues {
            break;
        }
        let dialogue = match draw_dialogue(world, &mut rng)? {
            Drawn::Ok(d) => d,
            Drawn::Redraw(reason) => {
                *tally.redraws.entry(reason.into()).or_default() += 1;
                redraws_total += 1;
                if redraws_total > 1000 + 50 * tally.dialogues {
                    return Err(format!("too many redraws: {:?}", tally.redraws));
                }
                continue;
            }
        };
        let messages: Vec<Message<'_>> = dialogue
            .turns
            .iter()
            .map(|t| Message {
                role: match t.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
                content: &t.text,
            })
            .collect();
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len()
            || encoded.special_token_occurrences != 0
            || encoded.tokens.len() != encoded.response_mask.len()
        {
            return Err(format!(
                "the encoder emitted {} of {} turns with {} special-token occurrences",
                encoded.emitted_turns,
                messages.len(),
                encoded.special_token_occurrences
            ));
        }
        if encoded.tokens.len() > enc.context {
            *tally.redraws.entry("over_context".into()).or_default() += 1;
            redraws_total += 1;
            continue;
        }
        if spec.token_budget > 0 && tally.tokens + encoded.tokens.len() > spec.token_budget {
            break;
        }
        let ids: Vec<u16> = encoded
            .tokens
            .iter()
            .map(|&t| u16::try_from(t).map_err(|_| "token id above u16".to_string()))
            .collect::<Result<_>>()?;
        // The masked region of each question's reply must decode to text that
        // holds the expected value: the trainer scores exactly those tokens.
        let runs = mask_runs(&encoded.response_mask);
        let assistant_turns: Vec<usize> = (0..dialogue.turns.len())
            .filter(|i| dialogue.turns[*i].role == Role::Assistant)
            .collect();
        if runs.len() != assistant_turns.len() {
            return Err(format!(
                "{} masked runs for {} assistant turns",
                runs.len(),
                assistant_turns.len()
            ));
        }
        for q in &dialogue.questions {
            let run = assistant_turns
                .iter()
                .position(|t| *t == q.turn)
                .ok_or("a question answer is not an assistant turn")?;
            let (start, end) = runs[run];
            tally.recall_response_tokens += end - start;
            if let Some(expect) = &q.expect {
                let scored: Vec<u32> = encoded.tokens[start..end].to_vec();
                let text = enc.tokenizer.decode(&scored);
                let got = content_stems(&text);
                if !content_stems(expect).iter().all(|s| got.contains(s)) {
                    return Err(format!(
                        "the scored reply {text:?} does not hold {expect:?}"
                    ));
                }
            }
        }
        writer.write_tokens(&ids).map_err(|e| e.to_string())?;
        mask_out
            .write_all(&encoded.response_mask)
            .map_err(|e| e.to_string())?;
        tally.tokens += ids.len();
        tally.response_tokens += encoded.response_mask.iter().filter(|&&m| m == 1).count();
        tally.max_tokens = tally.max_tokens.max(ids.len());
        if tally.dialogues < spec.samples {
            let id = format!("{}-{:07}", spec.name, tally.dialogues);
            serde_json::to_writer(&mut samples_out, &dialogue.record(&id))
                .map_err(|e| e.to_string())?;
            samples_out.write_all(b"\n").map_err(|e| e.to_string())?;
        }
        tally.add(&dialogue, &world.panel);
    }
    if tally.dialogues == 0 {
        return Err(format!("the {} split is empty", spec.name));
    }
    let written = writer.finish().map_err(|e| e.to_string())?;
    if written as usize != tally.tokens {
        return Err(format!("wrote {written} tokens, expected {}", tally.tokens));
    }
    mask_out.flush().map_err(|e| e.to_string())?;
    samples_out.flush().map_err(|e| e.to_string())?;
    drop(mask_out);
    drop(samples_out);

    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": "uor-r4-response-mask/u8/v1",
        "split": spec.name,
        "template_rule": "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.",
        "mask_rule": "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.",
        "dialogue_protocol": enc.protocol.schema,
        "drops": {
            "rows_dropped_no_messages": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": 0
        },
        "files": [{
            "label": spec.label,
            "tokens": tally.tokens,
            "response_tokens": tally.response_tokens,
            "special_token_occurrences": 0,
            "rows_used": tally.dialogues,
            "rows_total": tally.dialogues
        }],
        "max_tokens": enc.context,
        "rows_used": tally.dialogues,
        "tokens": tally.tokens,
        "response_tokens": tally.response_tokens,
        "response_fraction": tally.response_tokens as f64 / tally.tokens.max(1) as f64,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": fs::metadata(&mask_path).map_err(|e| e.to_string())?.len(),
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        "bos_id": enc.protocol.bos_id,
        "eos_id": enc.protocol.eos_id,
        "synthetic": {
            "generator": "dialogue-recall-corpus generate",
            "schema": SCHEMA,
            "seed": spec.seed,
            "counts": tally.json(),
        },
    });
    fs::write(
        spec.dir.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(tally)
}

/// (start, end) of every run of mask 1.
fn mask_runs(mask: &[u8]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (i, &m) in mask.iter().enumerate() {
        match (m, start) {
            (1, None) => start = Some(i),
            (0, Some(s)) => {
                runs.push((s, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        runs.push((s, mask.len()));
    }
    runs
}

// ---------------------------------------------------------------------------
// The decoded-store leak check.

#[derive(Default)]
struct StoreLeak {
    documents: usize,
    turns: usize,
    panel_turn_matches: usize,
    shared_6grams: usize,
    recall_value_word_hits: usize,
    examples: Vec<Value>,
    frame_words_shared: BTreeMap<String, usize>,
}

impl StoreLeak {
    fn json(&self) -> Value {
        let mut shared: Vec<(&String, &usize)> = self.frame_words_shared.iter().collect();
        shared.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        json!({
            "documents": self.documents,
            "turns_checked": self.turns,
            "panel_turn_matches": self.panel_turn_matches,
            "shared_6grams": self.shared_6grams,
            "recall_value_word_hits": self.recall_value_word_hits,
            "examples": self.examples,
            "frame_words_shared_with_panels_distinct": self.frame_words_shared.len(),
            "frame_words_shared_with_panels_top": shared.iter().take(80)
                .map(|(w, n)| json!([w, n])).collect::<Vec<_>>(),
            "pass": self.panel_turn_matches == 0 && self.shared_6grams == 0
                && self.recall_value_word_hits == 0,
        })
    }
}

/// Decode a prepared store document by document and check every turn.
fn store_leak(store: &Path, tokenizer: &ByteBpeTokenizer, panel: &Panel) -> Result<StoreLeak> {
    let reader = MmapCorpusReader::open(store.join("tokens.u16")).map_err(|e| e.to_string())?;
    let protocol = DialogueProtocol::literal_roles_v2(tokenizer).map_err(|e| e.to_string())?;
    let (bos, eos) = (protocol.bos_id, protocol.eos_id);
    let ids = reader.as_slice();
    let mut leak = StoreLeak::default();
    let mut start = 0usize;
    for end in 1..=ids.len() {
        if end < ids.len() && u32::from(ids[end]) != bos {
            continue;
        }
        let doc: Vec<u32> = ids[start..end]
            .iter()
            .map(|&id| u32::from(id))
            .filter(|&id| id != bos && id != eos)
            .collect();
        start = end;
        leak.documents += 1;
        let text = tokenizer.decode(&doc);
        for line in text.split('\n') {
            let content = ["User:", "Assistant:", "System:"]
                .iter()
                .find_map(|m| line.trim_start().strip_prefix(m))
                .unwrap_or(line)
                .trim();
            if content.is_empty() {
                continue;
            }
            leak.turns += 1;
            let mut hit = None;
            if panel.turns.contains(&normalized(content)) {
                leak.panel_turn_matches += 1;
                hit = Some("panel_turn");
            }
            let shared: Vec<String> = grams(content)
                .into_iter()
                .filter(|g| panel.grams.contains(g))
                .collect();
            if !shared.is_empty() {
                leak.shared_6grams += shared.len();
                hit = Some("panel_6gram");
            }
            let stems = content_stems(content);
            if stems.iter().any(|s| panel.strict.contains(s)) {
                leak.recall_value_word_hits += 1;
                hit = Some("panel_recall_value_word");
            }
            for s in stems {
                if panel.content.contains(&s) {
                    *leak.frame_words_shared.entry(s).or_default() += 1;
                }
            }
            if let Some(kind) = hit {
                let example = json!({"kind": kind, "turn": content});
                if leak.examples.len() < 40 && !leak.examples.contains(&example) {
                    leak.examples.push(example);
                }
            }
        }
    }
    Ok(leak)
}

// ---------------------------------------------------------------------------
// Driver.

fn arg<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.iter()
        .find_map(|a| a.strip_prefix(key).and_then(|r| r.strip_prefix('=')))
}

fn number(args: &[String], key: &str, default: u64) -> Result<u64> {
    match arg(args, key) {
        None => Ok(default),
        Some(v) => v
            .parse()
            .map_err(|_| format!("{key}= must be a whole number")),
    }
}

fn panel_dirs(args: &[String]) -> Vec<PathBuf> {
    arg(args, "panels")
        .unwrap_or("data/panels")
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

fn load_tokenizer(path: &str) -> Result<ByteBpeTokenizer> {
    let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| "unreadable tokenizer.json".into())
}

fn source_commit(args: &[String]) -> Value {
    if let Some(commit) = arg(args, "source_commit") {
        return json!({"commit": commit, "from": "source_commit="});
    }
    if let Some(commit) = option_env!("UOR_BUILD_SOURCE_COMMIT") {
        return json!({"commit": commit, "from": "UOR_BUILD_SOURCE_COMMIT"});
    }
    let git = |a: &[&str]| {
        std::process::Command::new("git")
            .args(a)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
    };
    match git(&["rev-parse", "HEAD"]) {
        Some(commit) => json!({
            "commit": commit,
            "from": "git rev-parse HEAD at run time",
            "dirty_including_untracked": git(&["status", "--porcelain"]).map(|s| !s.is_empty()),
        }),
        None => json!({"commit": "UNAVAILABLE", "from": "none"}),
    }
}

fn generate(args: &[String]) -> Result<()> {
    let out = PathBuf::from(arg(args, "out").ok_or("missing out=")?);
    let tokenizer_path = arg(args, "tokenizer")
        .ok_or("missing tokenizer=")?
        .to_owned();
    let seed = number(args, "seed", 1)?;
    let dialogues = number(args, "dialogues", 0)? as usize;
    let token_budget = number(args, "token_budget", 0)? as usize;
    if (dialogues == 0) == (token_budget == 0) {
        return Err("give exactly one of dialogues=N or token_budget=N".into());
    }
    let dev_seed = number(args, "dev_seed", 1_000_003)?;
    if dev_seed == seed {
        return Err("dev_seed must differ from seed".into());
    }
    let dev_dialogues = number(args, "dev_dialogues", 300)? as usize;
    let version = u8::try_from(number(args, "protocol", 2)?).map_err(|_| "protocol= is 1 or 2")?;
    if !(1..=2).contains(&version) {
        return Err("protocol= is 1 or 2".into());
    }
    let context = number(args, "context", 384)? as usize;
    if context < 256 {
        return Err("context= must be at least 256".into());
    }
    let samples = number(args, "samples", 200)? as usize;
    let dirs = panel_dirs(args);
    // Validate the inputs before claiming the root.
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let panel = Panel::load(&dirs)?;
    let world = World::new(panel)?;
    let protocol =
        DialogueProtocol::literal_roles_version(&tokenizer, version).map_err(|e| e.to_string())?;
    let vocab = u32::try_from(tokenizer.vocab_size()).map_err(|_| "vocabulary above u32")?;

    report_output::claim(&out).map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    let enc = Encoding {
        tokenizer: &tokenizer,
        protocol,
        version,
        context,
        vocab,
    };
    let train = write_split(
        &world,
        &enc,
        &SplitSpec {
            name: "train",
            label: "dialogue-recall",
            seed,
            dialogues,
            token_budget,
            samples,
            dir: &out.join("train"),
        },
    )?;
    let dev = write_split(
        &world,
        &enc,
        &SplitSpec {
            name: "dev",
            label: "dialogue-recall-dev",
            seed: dev_seed,
            dialogues: dev_dialogues,
            token_budget: 0,
            samples: usize::MAX,
            dir: &out.join("dev"),
        },
    )?;

    // Each split must load and index exactly as `dialogue-train` will read it.
    let mut populations = serde_json::Map::new();
    for name in ["train", "dev"] {
        let dir = out.join(name);
        let (_, contract) = episode_contract_for(&tokenizer, vocab as usize, context, enc.version)
            .map_err(|e| e.to_string())?;
        let split = DialogueSplit::load(
            &dir.join("tokens.u16"),
            &dir.join("response_mask.u8"),
            &dir.join("manifest.json"),
        )
        .map_err(|e| e.to_string())?;
        let index = split.index(contract).map_err(|e| e.to_string())?;
        let p = index.population();
        populations.insert(
            name.into(),
            json!({
                "documents": p.documents,
                "response_runs": p.response_runs,
                "eligible_responses": p.eligible_responses,
                "excluded_over_context": p.excluded_over_context,
                "context": context,
            }),
        );
    }

    // The leak report: decoded from the written stores.
    let train_leak = store_leak(&out.join("train"), &tokenizer, &world.panel)?;
    let dev_leak = store_leak(&out.join("dev"), &tokenizer, &world.panel)?;
    let structured = |t: &Tally| {
        json!({
            "recall_value_stems_shared_with_panels": t.value_stems_shared,
            "key_words_shared_with_panels": t.key_words_shared,
            "dialogues_with_a_panel_key_word": t.dialogues_with_panel_key,
            "strict_key_dialogues": t.strict_key_dialogues,
            "strict_key_share": t.strict_key_dialogues as f64 / t.dialogues.max(1) as f64,
        })
    };
    let pass = train_leak.panel_turn_matches == 0
        && train_leak.shared_6grams == 0
        && train_leak.recall_value_word_hits == 0
        && dev_leak.panel_turn_matches == 0
        && dev_leak.shared_6grams == 0
        && dev_leak.recall_value_word_hits == 0
        && train.value_stems_shared.is_empty()
        && dev.value_stems_shared.is_empty();
    let leak = json!({
        "schema": LEAK_SCHEMA,
        "pass": pass,
        "method": {
            "store": "each document of tokens.u16 decoded with the tokenizer, split into turns at the protocol's newline separator, role markers removed",
            "panel_turn_match": "a turn whose lowercased words (punctuation ignored) equal a panel string's",
            "shared_6grams": "word 6-grams of a turn found among the 6-grams of any panel string",
            "recall_value_word": "a content-word stem of a panel recall row's terms or forbid column (values, distractors and their closed classes)",
            "recall_value_stems_shared": "from the generator's records: stems of every stated value (and superseded value) found among the content stems of all panel strings and check fields",
            "frame_words": "content stems of generated turns that also occur in a panel; allowed and listed",
        },
        "panels": world.panel.files.iter().map(|(p, s)| json!({"path": p, "sha256": s})).collect::<Vec<_>>(),
        "panel_strings": world.panel.strings,
        "panel_6grams": world.panel.grams.len(),
        "panel_content_stems": world.panel.content.len(),
        "panel_recall_value_stems": world.panel.strict,
        "panel_recall_key_stems": world.panel.keys,
        "dropped_by_panel_filter": world.dropped,
        "train": {"decoded": train_leak.json(), "records": structured(&train)},
        "dev": {"decoded": dev_leak.json(), "records": structured(&dev)},
    });
    fs::write(
        out.join("leak.json"),
        serde_json::to_vec_pretty(&leak).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if !pass {
        return Err(format!(
            "the leak check failed; see {}",
            out.join("leak.json").display()
        ));
    }

    let mut files = serde_json::Map::new();
    for rel in [
        "train/tokens.u16",
        "train/response_mask.u8",
        "train/manifest.json",
        "train/samples.jsonl",
        "dev/tokens.u16",
        "dev/response_mask.u8",
        "dev/manifest.json",
        "dev/dialogues.jsonl",
        "leak.json",
    ] {
        let path = out.join(rel);
        files.insert(
            rel.into(),
            json!({
                "sha256": uor_r4_training::sha256_file(&path).map_err(|e| e.to_string())?,
                "bytes": fs::metadata(&path).map_err(|e| e.to_string())?.len(),
            }),
        );
    }
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let generator = json!({
        "schema": SCHEMA,
        "argv": args,
        "source": source_commit(args),
        "executable": {
            "path": executable.display().to_string(),
            "sha256": uor_r4_training::sha256_file(&executable).ok(),
        },
        "tokenizer": {
            "path": tokenizer_path,
            "sha256": uor_r4_training::sha256_file(Path::new(&tokenizer_path)).map_err(|e| e.to_string())?,
        },
        "dialogue_protocol": enc.protocol.schema,
        "context": context,
        "seed": seed,
        "dev_seed": dev_seed,
        "requested": {"dialogues": dialogues, "token_budget": token_budget, "dev_dialogues": dev_dialogues},
        "train": train.json(),
        "dev": dev.json(),
        "populations": populations,
        "leak_pass": pass,
        "relations": FRAMES.iter().map(|f| f.id).collect::<Vec<_>>(),
        "pool_sizes": world.values.iter().map(|(k, v)| (format!("{k:?}"), v.len())).collect::<BTreeMap<_, _>>(),
        "primary_category_shares": PRIMARY_SHARES.iter().map(|(c, s)| (c.name(), *s)).collect::<BTreeMap<_, _>>(),
        "followup_abstain_probability": FOLLOWUP_ABSTAIN,
        "strict_key_share_target": STRICT_KEY_SHARE,
        "files": files,
        "seconds": started.elapsed().as_secs_f64(),
        "scope": "training data for in-context recall in natural dialogue; not an evaluation. The dev split shares pools and templates with train and is for monitoring only.",
    });
    fs::write(
        out.join("generator.json"),
        serde_json::to_vec_pretty(&generator).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    report_output::seal(&out).map_err(|e| e.to_string())?;
    report_output::verify(&out).map_err(|e| e.to_string())?;
    println!(
        "train: {} dialogues, {} tokens ({} recall-answer tokens); dev: {} dialogues, {} tokens; leak pass",
        train.dialogues, train.tokens, train.recall_response_tokens, dev.dialogues, dev.tokens
    );
    Ok(())
}

fn leak_mode(args: &[String]) -> Result<()> {
    let out = PathBuf::from(arg(args, "out").ok_or("missing out=")?);
    let store = PathBuf::from(arg(args, "store").ok_or("missing store=")?);
    let tokenizer_path = arg(args, "tokenizer").ok_or("missing tokenizer=")?;
    let tokenizer = load_tokenizer(tokenizer_path)?;
    let panel = Panel::load(&panel_dirs(args))?;
    report_output::claim(&out).map_err(|e| e.to_string())?;
    let leak = store_leak(&store, &tokenizer, &panel)?;
    let report = json!({
        "schema": LEAK_SCHEMA,
        "store": store.display().to_string(),
        "tokens_sha256": uor_r4_training::sha256_file(&store.join("tokens.u16")).map_err(|e| e.to_string())?,
        "panels": panel.files.iter().map(|(p, s)| json!({"path": p, "sha256": s})).collect::<Vec<_>>(),
        "panel_strings": panel.strings,
        "panel_6grams": panel.grams.len(),
        "decoded": leak.json(),
    });
    fs::write(
        out.join("leak.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    report_output::seal(&out).map_err(|e| e.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report["decoded"]).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn run(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("generate") => generate(&args[1..]),
        Some("leak") => leak_mode(&args[1..]),
        _ => Err(
            "usage: dialogue-recall-corpus generate|leak key=value ... (see the module docs)"
                .into(),
        ),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = run(&args) {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> World {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/panels");
        let panel = Panel::load(&[dir]).expect("the frozen panels load");
        World::new(panel).expect("the world builds")
    }

    fn draw_many(world: &World, seed: u64, n: usize) -> Vec<Dialogue> {
        let mut rng = Rng::new(seed);
        let mut out = Vec::new();
        while out.len() < n {
            match draw_dialogue(world, &mut rng).expect("no generator error") {
                Drawn::Ok(d) => out.push(d),
                Drawn::Redraw(_) => {}
            }
        }
        out
    }

    fn jsonl(dialogues: &[Dialogue]) -> String {
        dialogues
            .iter()
            .enumerate()
            .map(|(i, d)| d.record(&i.to_string()).to_string() + "\n")
            .collect()
    }

    #[test]
    fn same_seed_gives_identical_bytes_and_another_seed_differs() {
        let w = world();
        let a = jsonl(&draw_many(&w, 7, 400));
        let b = jsonl(&draw_many(&w, 7, 400));
        let c = jsonl(&draw_many(&w, 8, 400));
        assert_eq!(a, b);
        assert_ne!(a, c);
        // A second world from the same panels gives the same bytes too.
        let w2 = world();
        assert_eq!(a, jsonl(&draw_many(&w2, 7, 400)));
    }

    #[test]
    fn answers_name_the_bound_value_and_never_a_distractor() {
        let w = world();
        let dialogues = draw_many(&w, 11, 3000);
        let mut with_distractor = 0;
        for d in &dialogues {
            check_dialogue(d).expect("every question passes its own check");
            for q in d.questions.iter().filter(|q| q.expect.is_some()) {
                let answer = content_stems(&d.turns[q.turn].text);
                let expect = q.expect.as_deref().unwrap_or_default();
                assert!(content_stems(expect).iter().all(|s| answer.contains(s)));
                for v in &q.forbid {
                    assert!(content_stems(v).iter().all(|s| !answer.contains(s)));
                }
                if !q.forbid.is_empty() {
                    with_distractor += 1;
                    // Each distractor is stated before the question.
                    let earlier: BTreeSet<String> = d.turns[..q.turn - 1]
                        .iter()
                        .flat_map(|t| content_stems(&t.text))
                        .collect();
                    for v in &q.forbid {
                        assert!(content_stems(v).iter().all(|s| earlier.contains(s)), "{v}");
                    }
                }
            }
        }
        assert!(with_distractor > 2000, "{with_distractor}");
    }

    #[test]
    fn binding_rows_state_two_same_type_keys_in_one_turn() {
        let w = world();
        let mut seen = 0;
        for d in draw_many(&w, 13, 2000)
            .iter()
            .filter(|d| d.category == Category::Binding)
        {
            let q = &d.questions[0];
            let expect = q.expect.as_deref().unwrap_or_default();
            // The turn stating the asked value also states a same-relation distractor.
            let turn = d
                .turns
                .iter()
                .take(q.turn - 1)
                .find(|t| {
                    t.role == Role::User
                        && content_stems(&t.text).is_superset(&content_stems(expect))
                })
                .expect("the value is stated in a user turn");
            if q.forbid
                .iter()
                .any(|v| content_stems(&turn.text).is_superset(&content_stems(v)))
            {
                seen += 1;
            }
        }
        assert!(seen > 200, "{seen}");
    }

    #[test]
    fn the_latest_value_wins_after_an_update() {
        let w = world();
        let mut latest = 0;
        let mut unchanged = 0;
        for d in draw_many(&w, 17, 4000) {
            for q in d
                .questions
                .iter()
                .filter(|q| q.category == Category::Update)
            {
                let expect = q.expect.clone().unwrap_or_default();
                let fact = d
                    .facts
                    .iter()
                    .find(|f| f.value == expect)
                    .expect("the expected value is a current fact");
                if q.detail == "latest" {
                    latest += 1;
                    let old = fact
                        .old
                        .clone()
                        .expect("an updated fact keeps its old value");
                    assert!(q.forbid.contains(&old));
                    // The old value is stated, then the new value, in that order.
                    let first = |v: &str| {
                        d.turns
                            .iter()
                            .position(|t| content_stems(&t.text).is_superset(&content_stems(v)))
                            .unwrap_or(usize::MAX)
                    };
                    assert!(first(&old) < first(&expect));
                    assert!(first(&expect) < q.turn);
                } else {
                    unchanged += 1;
                    assert!(fact.old.is_none());
                    assert!(d
                        .facts
                        .iter()
                        .any(|f| f.old.as_ref().is_some_and(|o| q.forbid.contains(o))));
                }
            }
        }
        assert!(latest > 300 && unchanged > 50, "{latest} {unchanged}");
    }

    #[test]
    fn abstentions_name_no_stated_value_and_ask_an_unstated_key() {
        let w = world();
        let dialogues = draw_many(&w, 19, 4000);
        let mut abstain = 0;
        let mut questions = 0;
        for d in &dialogues {
            for q in &d.questions {
                questions += 1;
                if q.expect.is_some() {
                    continue;
                }
                abstain += 1;
                let answer = content_stems(&d.turns[q.turn].text);
                for fact in &d.facts {
                    for v in std::iter::once(&fact.value).chain(fact.old.iter()) {
                        assert!(content_stems(v).iter().all(|s| !answer.contains(s)));
                    }
                }
                assert!(
                    d.turns[q.turn].text.contains("haven't")
                        || d.turns[q.turn].text.contains("didn't")
                        || d.turns[q.turn].text.contains("never")
                );
            }
        }
        let share = abstain as f64 / questions as f64;
        assert!((0.11..=0.20).contains(&share), "abstention share {share}");
    }

    #[test]
    fn no_panel_turn_no_shared_6gram_and_no_panel_value_word() {
        let w = world();
        let dialogues = draw_many(&w, 23, 5000);
        for d in &dialogues {
            for t in &d.turns {
                assert!(!w.panel.turns.contains(&normalized(&t.text)), "{}", t.text);
                for g in grams(&t.text) {
                    assert!(!w.panel.grams.contains(&g), "{g}");
                }
                assert!(!w.panel.has_strict(&t.text), "{}", t.text);
            }
            for fact in &d.facts {
                assert!(!w.panel.shares_content(&fact.value), "{}", fact.value);
            }
        }
        // The panel's own memory example values are all excluded.
        for word in [
            "shelby", "flash", "plums", "pears", "grandpa", "turtle", "saturday",
        ] {
            assert!(w.panel.content.contains(&stem(word)), "{word}");
        }
        for word in [
            "shelby", "flash", "plum", "pear", "saturday", "blue", "nine",
        ] {
            assert!(w.panel.strict.contains(&stem(word)), "{word}");
        }
        // Strict-key dialogues use no panel key word; at least half are strict.
        let strict = dialogues.iter().filter(|d| d.strict_keys).count();
        assert!(strict * 2 > dialogues.len());
        for d in dialogues.iter().filter(|d| d.strict_keys) {
            for k in &d.keys {
                assert!(!w.panel.shares_content(&k.user), "{}", k.user);
            }
        }
    }

    #[test]
    fn the_leak_check_detects_an_injected_panel_turn() {
        let w = world();
        assert_eq!(
            w.panel.turn_leak("Which name did we give the turtle?"),
            Some("panel_turn")
        );
        assert_eq!(
            w.panel.turn_leak(
                "Yesterday we got two new pets this week. The turtle is Shelby and the goldfish is Flash."
            ),
            Some("panel_6gram")
        );
        assert_eq!(
            w.panel.turn_leak("The cat is named Shelby."),
            Some("panel_recall_value_word")
        );
        assert_eq!(w.panel.turn_leak("The ferret is named Quill."), None);
    }

    #[test]
    fn placeholders_fill_or_fail() {
        let mut slots = BTreeMap::new();
        slots.insert("k".to_owned(), "the gecko".to_owned());
        assert_eq!(
            fill("Where is {k}?", &slots).as_deref(),
            Ok("Where is the gecko?")
        );
        assert!(fill("Where is {v}?", &slots).is_err());
        assert_eq!(article("umbrella", Art::Indef), "an umbrella");
        assert_eq!(article("lantern", Art::Indef), "a lantern");
        assert_eq!(stem("tomatoes"), "tomato");
        assert_eq!(stem("goldfish's".replace('\'', "").as_str()), "goldfish");
    }
}
