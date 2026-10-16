atom capacity q=82; nested steps n=16; absorb steps k=3

| corpus | input B | E(x) B | ratio | dict B | verbatim B | leaves (distinct/reused) | parents (distinct/reused) | carries | round-trip | Q0(n=16) B | gzip -9 | xz -9 | zstd -19 | H0 bits |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|---:|---:|---:|
| uniform_random | 65536 | 69827 | 1.0655 | 69783 | 65536 | 800/0 | 396/0 | 396 | exact | 193927 | 65574 | 65600 | 65549 | 524115 |
| zeros | 65536 | 164 | 0.0025 | 129 | 100 | 2/798 | 6/390 | 396 | exact | 732 | 96 | 140 | 22 | 0 |
| cycle_period_82 | 65536 | 164 | 0.0025 | 129 | 100 | 2/798 | 6/390 | 396 | exact | 732 | 383 | 232 | 105 | 400667 |
| cycle_period_96 | 65536 | 4332 | 0.0661 | 4297 | 3954 | 49/751 | 61/335 | 396 | exact | 9669 | 394 | 248 | 119 | 405095 |
| cycle_period_1 | 65536 | 164 | 0.0025 | 129 | 100 | 2/798 | 6/390 | 396 | exact | 732 | 97 | 140 | 22 | 0 |
| structured_records | 65536 | 68985 | 1.0526 | 68941 | 64716 | 790/10 | 396/0 | 396 | exact | 191528 | 2394 | 1128 | 1281 | 281981 |
| low_entropy_4symbols | 65536 | 69827 | 1.0655 | 69783 | 65536 | 800/0 | 396/0 | 396 | exact | 193927 | 19750 | 18544 | 16535 | 131069 |
| base64_alphabet | 65536 | 69827 | 1.0655 | 69783 | 65536 | 800/0 | 396/0 | 396 | exact | 193927 | 49652 | 50444 | 49249 | 393163 |
| all_byte_values | 65536 | 12285 | 0.1875 | 12245 | 10514 | 129/671 | 258/138 | 396 | exact | 31373 | 597 | 372 | 280 | 524288 |
| AGENTS.md | 46272 | 49269 | 1.0648 | 49223 | 46272 | 565/0 | 278/0 | 278 | exact | 135803 | 18354 | 17320 | 17616 | 221270 |
| Cargo.lock | 74318 | 79115 | 1.0645 | 79073 | 74236 | 906/1 | 450/0 | 450 | exact | 220126 | 17816 | 15724 | 15752 | 388707 |
| README.md | 50606 | 53896 | 1.0650 | 53852 | 50606 | 618/0 | 305/0 | 305 | exact | 148914 | 16561 | 15464 | 15867 | 262475 |
| DECISIONS.md | 125024 | 133304 | 1.0662 | 133262 | 125024 | 1525/0 | 759/0 | 759 | exact | 373117 | 47078 | 42908 | 43732 | 600223 |
| spiralcore_operator.rs | 53471 | 56871 | 1.0636 | 56829 | 53389 | 652/1 | 323/0 | 323 | exact | 157374 | 11398 | 10568 | 10691 | 257488 |
| pdf_text.txt | 14100 | 14942 | 1.0597 | 14908 | 14100 | 172/0 | 84/0 | 84 | exact | 38941 | 5840 | 5664 | 5672 | 69529 |

uniform_random: nested round-trip exact | absorb k=3 sizes [74766, 80032, 85663] round-trip exact | sha256(in)=6c4a294bdb73b028607b125d642b756d8c38a0e21df162e33f2ad3bb1b121145 sha256(E(x))=1e3fe2cd69f1c9929d7e742f66dad01746db46dceafdcfceaf945623387ff58e
zeros: nested round-trip exact | absorb k=3 sizes [552, 956, 1379] round-trip exact | sha256(in)=de2f256064a0af797747c2b97505dc0b9f3df0de4f489eac731c23ae9ca9cc31 sha256(E(x))=f362401f2ca0e94e2c7981aa6d672320f6fb73eb35cf7bc2f123d1dd78f04478
cycle_period_82: nested round-trip exact | absorb k=3 sizes [552, 956, 1379] round-trip exact | sha256(in)=d7a191acc4f244ca213635a956b0009ec3ca65b9be687b61948bb6ba89aad410 sha256(E(x))=5b0ad74e168e709d6ff120618e2333e335f48ee504f11a50b54ccf18933efc2e
cycle_period_96: nested round-trip exact | absorb k=3 sizes [4920, 5538, 6185] round-trip exact | sha256(in)=098f10114ea6522756d0d9c320389b8f7be1144a7e65ae06e689cef7369dbc1a sha256(E(x))=10d108bcdf95271d8d168dac8951d0babf6cb229603bc6dfb995f40b581f50f4
cycle_period_1: nested round-trip exact | absorb k=3 sizes [552, 956, 1379] round-trip exact | sha256(in)=156c38442089c1323d3e3ba549a6ac24341c47e8b6367bec4740c9b8c865826e sha256(E(x))=bf1b00c8079dd980b4917e53c78d2b66a4b1d96ca53da55128fd720aee9f19b9
structured_records: nested round-trip exact | absorb k=3 sizes [73872, 79079, 84635] round-trip exact | sha256(in)=a590d86039526703b81fe7f6d091d8c079853d248eef7317fe9db1abe97d43aa sha256(E(x))=55e873b5b971cbe2bd1998ea3ed986db7ed654503b8fee0ed59b430c8299a76c
low_entropy_4symbols: nested round-trip exact | absorb k=3 sizes [74766, 80032, 85663] round-trip exact | sha256(in)=effca359983b341ead19603ec77ad91a9694b148f4b650e5d841728db60f0e15 sha256(E(x))=a81631b63734e5f7961d83c829cf64656aa497eb682c4e0532367341b5886486
base64_alphabet: nested round-trip exact | absorb k=3 sizes [74766, 80032, 85663] round-trip exact | sha256(in)=1542cc5095b1e9cf4cf9686bbf7f120f14b49b48fde507f4d984864bc1fe6b6b sha256(E(x))=2735355955ba2d86a94df6f6abdfee7d4717c35609aceb5c5a9d41e6e464078a
all_byte_values: nested round-trip exact | absorb k=3 sizes [13361, 14517, 15747] round-trip exact | sha256(in)=7daca2095d0438260fa849183dfc67faa459fdf4936e1bc91eec6b281b27e4c2 sha256(E(x))=79280d1ed5c249b198264d1845254bd217cb419fa3c07bf6d7401a677895a8ce
AGENTS.md: nested round-trip exact | absorb k=3 sizes [52826, 56628, 60677] round-trip exact | sha256(in)=f9f13a95bd16b3e7134eb95b8eec34c3c17a37e9ef9ee4124b4918cde69837d7 sha256(E(x))=f9fa4fcc42e8bb6e7b141f5b364374a047f56fa5f0457192566f54d262a706c6
Cargo.lock: nested round-trip exact | absorb k=3 sizes [84671, 90604, 96940] round-trip exact | sha256(in)=d1471fef2f412d28a4d729756bf533756e31ccb544a98365591e851c3d3f870b sha256(E(x))=e91c17a51cc86fdd23319177223a41634605cc5e769db6d40182e7fc5bc9f341
README.md: nested round-trip exact | absorb k=3 sizes [57765, 61891, 66301] round-trip exact | sha256(in)=95889609589bb0ec6d0b4c962e029beebbc43bb2fa20939974697f3fceb4b7ee sha256(E(x))=0edbd2399e9a028e4401d470291d5f90e1fdcb367db0a3d128c03b114ee069bd
DECISIONS.md: nested round-trip exact | absorb k=3 sizes [142500, 152309, 162772] round-trip exact | sha256(in)=11c6148ebc26662075edad3dbd1bd8c25c74e97d5531a7bf1e8febc54c7dff04 sha256(E(x))=7249083019bfdc94d7d7934d99e256a84658349bc7fb8aa51f002c0156d23a70
spiralcore_operator.rs: nested round-trip exact | absorb k=3 sizes [60935, 65283, 69921] round-trip exact | sha256(in)=1e17032031620e87d705b56a4529f4d04488a613449ec7cb0a9a7ccb862e744b sha256(E(x))=5cf5cdc82098dc03242c22c9e5a164e26770d89c8980cb94ae95a01fbd2d86cb
pdf_text.txt: nested round-trip exact | absorb k=3 sizes [16198, 17539, 18969] round-trip exact | sha256(in)=5a60e759a1b197d03e4978986ab61c9ff46f820739c87ed1f87d22d7a9b61b60 sha256(E(x))=2df680b1ae3b191924dbb95fa46027219be8c7678246ea0f3e222c3582472d70

wrote /Users/casey.allard/uor-r4/.worktrees/deepseek-spiralcore-fbs-replay/docs/labs/spiralcore-recursive-fbs-2026-10-16/evidence/setA-64KiB/receipt.json
