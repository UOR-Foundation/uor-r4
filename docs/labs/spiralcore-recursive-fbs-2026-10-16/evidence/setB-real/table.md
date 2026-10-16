atom capacity q=82; nested steps n=2; absorb steps k=1

| corpus | input B | E(x) B | ratio | dict B | verbatim B | leaves (distinct/reused) | parents (distinct/reused) | carries | round-trip | Q0(n=2) B | gzip -9 | xz -9 | zstd -19 | H0 bits |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|---:|---:|
| uniform_random | 65536 | 69827 | 1.0655 | 69783 | 65536 | 800/0 | 396/0 | 396 | exact | 79288 | 65574 | 65600 | 65549 | 524115 |
| zeros | 65536 | 164 | 0.0025 | 129 | 100 | 2/798 | 6/390 | 396 | exact | 230 | 96 | 140 | 22 | 0 |
| cycle_period_82 | 65536 | 164 | 0.0025 | 129 | 100 | 2/798 | 6/390 | 396 | exact | 230 | 383 | 232 | 105 | 400667 |
| cycle_period_96 | 65536 | 4332 | 0.0661 | 4297 | 3954 | 49/751 | 61/335 | 396 | exact | 4808 | 394 | 248 | 119 | 405095 |
| cycle_period_1 | 65536 | 164 | 0.0025 | 129 | 100 | 2/798 | 6/390 | 396 | exact | 230 | 97 | 140 | 22 | 0 |
| structured_records | 65536 | 68985 | 1.0526 | 68941 | 64716 | 790/10 | 396/0 | 396 | exact | 78328 | 2394 | 1128 | 1281 | 281981 |
| low_entropy_4symbols | 65536 | 69827 | 1.0655 | 69783 | 65536 | 800/0 | 396/0 | 396 | exact | 79288 | 19750 | 18544 | 16535 | 131069 |
| base64_alphabet | 65536 | 69827 | 1.0655 | 69783 | 65536 | 800/0 | 396/0 | 396 | exact | 79288 | 49652 | 50444 | 49249 | 393163 |
| all_byte_values | 65536 | 12285 | 0.1875 | 12245 | 10514 | 129/671 | 258/138 | 396 | exact | 13762 | 597 | 372 | 280 | 524288 |
| corpus-docs-4MiB.txt | 4194304 | 4531602 | 1.0804 | 4531539 | 4190204 | 51101/50 | 25564/6 | 25570 | exact | 5303914 | 1341837 | 982580 | 1022754 | 21567865 |
| corpus-core-src.tar | 13998080 | 14858514 | 1.0615 | 14858453 | 13694434 | 167006/3703 | 84948/401 | 85349 | exact | 17469108 | 2976711 | 1944060 | 1980390 | 66275447 |

uniform_random: nested round-trip exact | absorb k=1 sizes [74766] round-trip exact | sha256(in)=6c4a294bdb73b028607b125d642b756d8c38a0e21df162e33f2ad3bb1b121145 sha256(E(x))=1e3fe2cd69f1c9929d7e742f66dad01746db46dceafdcfceaf945623387ff58e
zeros: nested round-trip exact | absorb k=1 sizes [552] round-trip exact | sha256(in)=de2f256064a0af797747c2b97505dc0b9f3df0de4f489eac731c23ae9ca9cc31 sha256(E(x))=f362401f2ca0e94e2c7981aa6d672320f6fb73eb35cf7bc2f123d1dd78f04478
cycle_period_82: nested round-trip exact | absorb k=1 sizes [552] round-trip exact | sha256(in)=d7a191acc4f244ca213635a956b0009ec3ca65b9be687b61948bb6ba89aad410 sha256(E(x))=5b0ad74e168e709d6ff120618e2333e335f48ee504f11a50b54ccf18933efc2e
cycle_period_96: nested round-trip exact | absorb k=1 sizes [4920] round-trip exact | sha256(in)=098f10114ea6522756d0d9c320389b8f7be1144a7e65ae06e689cef7369dbc1a sha256(E(x))=10d108bcdf95271d8d168dac8951d0babf6cb229603bc6dfb995f40b581f50f4
cycle_period_1: nested round-trip exact | absorb k=1 sizes [552] round-trip exact | sha256(in)=156c38442089c1323d3e3ba549a6ac24341c47e8b6367bec4740c9b8c865826e sha256(E(x))=bf1b00c8079dd980b4917e53c78d2b66a4b1d96ca53da55128fd720aee9f19b9
structured_records: nested round-trip exact | absorb k=1 sizes [73872] round-trip exact | sha256(in)=a590d86039526703b81fe7f6d091d8c079853d248eef7317fe9db1abe97d43aa sha256(E(x))=55e873b5b971cbe2bd1998ea3ed986db7ed654503b8fee0ed59b430c8299a76c
low_entropy_4symbols: nested round-trip exact | absorb k=1 sizes [74766] round-trip exact | sha256(in)=effca359983b341ead19603ec77ad91a9694b148f4b650e5d841728db60f0e15 sha256(E(x))=a81631b63734e5f7961d83c829cf64656aa497eb682c4e0532367341b5886486
base64_alphabet: nested round-trip exact | absorb k=1 sizes [74766] round-trip exact | sha256(in)=1542cc5095b1e9cf4cf9686bbf7f120f14b49b48fde507f4d984864bc1fe6b6b sha256(E(x))=2735355955ba2d86a94df6f6abdfee7d4717c35609aceb5c5a9d41e6e464078a
all_byte_values: nested round-trip exact | absorb k=1 sizes [13361] round-trip exact | sha256(in)=7daca2095d0438260fa849183dfc67faa459fdf4936e1bc91eec6b281b27e4c2 sha256(E(x))=79280d1ed5c249b198264d1845254bd217cb419fa3c07bf6d7401a677895a8ce
corpus-docs-4MiB.txt: nested round-trip exact | absorb k=1 sizes [4902311] round-trip exact | sha256(in)=c91ce57c3d3967639e5eb2fb43927cf7dd5dad623357b0068b6e9ecb78194b0b sha256(E(x))=ed14ef0ab07cb3a99e2c536c76a42a4a03ab784d1c9f6d706534f7885de1b125
corpus-core-src.tar: nested round-trip exact | absorb k=1 sizes [16110801] round-trip exact | sha256(in)=5aaf7c4176b9a3e935eb1dc7c6b9a9327762069a2f27cac70a1dc406293e818b sha256(E(x))=eef26094844c2d73827afc13ebb7f2f682f66cbc64f747329c1a2fef93aecbd3

wrote docs/labs/spiralcore-recursive-fbs-2026-10-16/evidence/setB-real/receipt.json
