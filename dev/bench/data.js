window.BENCHMARK_DATA = {
  "lastUpdate": 1780731897754,
  "repoUrl": "https://github.com/ai-screams/scoop-uv",
  "entries": {
    "scoop-uv benchmarks": [
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7f22a4344e66d0bb033505f5605062076513f558",
          "message": "Merge pull request #118 from ai-screams/feat/test-infra\n\nfeat(test-infra): devcontainer + multi-source matrix + Criterion bench gate",
          "timestamp": "2026-06-01T15:43:07+09:00",
          "tree_id": "27bd82801d7d0d19f8fe2c789321739fccabcdea",
          "url": "https://github.com/ai-screams/scoop-uv/commit/7f22a4344e66d0bb033505f5605062076513f558"
        },
        "date": 1780296438868,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 63941,
            "range": "± 474",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 75600,
            "range": "± 362",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3359,
            "range": "± 64",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 835,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1687,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1087,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 104,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 109,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 213,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "bcd380067813b0a381088d8b07832613601ab2d2",
          "message": "Merge pull request #119 from ai-screams/release-plz-2026-06-01T05-45-04Z\n\nchore: release v0.11.0",
          "timestamp": "2026-06-01T15:55:54+09:00",
          "tree_id": "6d81ddf3a44e4c95f4a28fff1af9ee3424eebe81",
          "url": "https://github.com/ai-screams/scoop-uv/commit/bcd380067813b0a381088d8b07832613601ab2d2"
        },
        "date": 1780297172525,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 67847,
            "range": "± 997",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 80269,
            "range": "± 1054",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3101,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 812,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1579,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 987,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 100,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 97,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 86,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 24,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 195,
            "range": "± 1",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "804cd8bb1f91b5cbd870debf4dd69e5dd468d9e3",
          "message": "Merge pull request #120 from ai-screams/dependabot/cargo/rust-dependencies-ae9fbc9046\n\nchore(deps): bump criterion from 0.5.1 to 0.7.0 in the rust-dependencies group across 1 directory",
          "timestamp": "2026-06-01T16:43:40+09:00",
          "tree_id": "c7d84393352b79d0a3f18bd8d054054cb017af41",
          "url": "https://github.com/ai-screams/scoop-uv/commit/804cd8bb1f91b5cbd870debf4dd69e5dd468d9e3"
        },
        "date": 1780300024754,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 64208,
            "range": "± 335",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 76059,
            "range": "± 961",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3186,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 856,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1680,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1127,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 105,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 92,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 209,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0a84b440f3b46cdb6f3a0472404ce590773ab13d",
          "message": "Merge pull request #121 from ai-screams/dependabot/github_actions/github-actions-29f2b1c28f\n\nci(deps): bump the github-actions group across 1 directory with 3 updates",
          "timestamp": "2026-06-01T16:44:18+09:00",
          "tree_id": "5480149ecc4c44d927ce29c4e57ee6bb75762cf4",
          "url": "https://github.com/ai-screams/scoop-uv/commit/0a84b440f3b46cdb6f3a0472404ce590773ab13d"
        },
        "date": 1780300235375,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 68735,
            "range": "± 682",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 81639,
            "range": "± 571",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3107,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 846,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1604,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1011,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 191,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "808ce80c413e2d2d8d373efa6711e85f020f11da",
          "message": "Merge pull request #122 from ai-screams/fix/venvwrapper-entrypoint-bypass\n\nfix(ci): bypass image entrypoint in matrix integration tests",
          "timestamp": "2026-06-01T17:05:52+09:00",
          "tree_id": "9aef906a9428fb9bb00abe42608ab75ed56a64c2",
          "url": "https://github.com/ai-screams/scoop-uv/commit/808ce80c413e2d2d8d373efa6711e85f020f11da"
        },
        "date": 1780301352296,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 52912,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 61684,
            "range": "± 359",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2510,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 638,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1305,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 847,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 79,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 83,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 17,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 163,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "pignuante",
            "username": "pignuante"
          },
          "committer": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "pignuante",
            "username": "pignuante"
          },
          "distinct": true,
          "id": "523b05bfd1b16556be0bcc900d6707f2591b092a",
          "message": "docs: sync v0.11.0 across user manuals and LLM exports\n\n- CHANGELOG: credit #116/#117/#118/#120/#121/#122 in v0.11.0 entry\n  and fix [Unreleased] compare base (v0.7.0 → 0.11.0)\n- README, installation, api: bump stale version references to 0.11.0\n- testing: refresh test counts to 751 (685 unit + 45 integration + 21 doctest)\n- quick-start: demonstrate `create --install-python`\n- python-management: note rayon parallelism for `migrate all`\n- llms.md / llms.txt: add 7 v0.11.0 commands to command tables\n- llms-full.txt: add Project Manifest + Collaboration sections,\n  5 new ScoopError variants, architecture entries for manifest.rs\n  and export_schema.rs, locale key count refresh\n- context7.json: add 6 LLM rules for .scoop.toml/sync/run/status/\n  which/export/import/clone/--install-python/rayon",
          "timestamp": "2026-06-01T17:44:12+09:00",
          "tree_id": "09b1162d4d323e2a8b41bcc4230f5acc5462321a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/523b05bfd1b16556be0bcc900d6707f2591b092a"
        },
        "date": 1780304028970,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 63875,
            "range": "± 1194",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 75367,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3224,
            "range": "± 47",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 858,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1698,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1083,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 104,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 107,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 96,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 211,
            "range": "± 1",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "510a90604a41c8f8ea587521ce05df3bd84cc59a",
          "message": "Merge pull request #123 from ai-screams/feat/gc-prune-man-verify\n\nfeat: 4 new commands (gc/prune/man/verify) + venvwrapper CI fix + reviews",
          "timestamp": "2026-06-02T11:58:14+09:00",
          "tree_id": "8027a495c1bfaf388909e273eacb5799fdfa1ca3",
          "url": "https://github.com/ai-screams/scoop-uv/commit/510a90604a41c8f8ea587521ce05df3bd84cc59a"
        },
        "date": 1780369307977,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 77808,
            "range": "± 811",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93018,
            "range": "± 1144",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3014,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 869,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1574,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 994,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 100,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 86,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 192,
            "range": "± 2",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0aa36baa9b3bee7ff062e92539ee483f1d79f9ca",
          "message": "Merge pull request #124 from ai-screams/release-plz-2026-06-02T02-58-54Z\n\nchore: release v0.12.0",
          "timestamp": "2026-06-02T13:53:09+09:00",
          "tree_id": "770845afbb6a8297d832e6342c247f2595c4efcf",
          "url": "https://github.com/ai-screams/scoop-uv/commit/0aa36baa9b3bee7ff062e92539ee483f1d79f9ca"
        },
        "date": 1780376198307,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 70955,
            "range": "± 320",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 82618,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3208,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 825,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1702,
            "range": "± 51",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1100,
            "range": "± 46",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 98,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 23,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 209,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "6b982ea7a6553e804fc28dc85aafa1ad85e873dd",
          "message": "Merge pull request #125 from ai-screams/feat/metadata-last-used\n\nfeat(core): metadata.last_used + status/list display + gc --older-than",
          "timestamp": "2026-06-02T19:09:47+09:00",
          "tree_id": "6bc33c622147919ebd040fa282e69b1fa659304d",
          "url": "https://github.com/ai-screams/scoop-uv/commit/6b982ea7a6553e804fc28dc85aafa1ad85e873dd"
        },
        "date": 1780395198469,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 71496,
            "range": "± 681",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 83985,
            "range": "± 749",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3256,
            "range": "± 142",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 822,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1693,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1086,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 104,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 104,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 96,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 212,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4eae07cf718f3cda73cd181581836ceae69358e8",
          "message": "Merge pull request #126 from ai-screams/release-plz-2026-06-02T10-10-46Z\n\nchore: release v0.13.0",
          "timestamp": "2026-06-02T20:17:55+09:00",
          "tree_id": "aa46d552fc75412b54be6d6fcdf2cb2458aa5231",
          "url": "https://github.com/ai-screams/scoop-uv/commit/4eae07cf718f3cda73cd181581836ceae69358e8"
        },
        "date": 1780399296816,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 76796,
            "range": "± 386",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 91508,
            "range": "± 2115",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3035,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 864,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1587,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1010,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 105,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 111,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 22,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 201,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4b96402fb860ddbfd01730bf44be87c8bb2a664d",
          "message": "Merge pull request #127 from ai-screams/feat/exit-status-layer\n\nfeat(v0.14): exit-status layer + migrate/diff commands + Korean docs (A-line)",
          "timestamp": "2026-06-06T14:33:39+09:00",
          "tree_id": "d293fb8e75dca98565c66795f24a0a9131a1512a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/4b96402fb860ddbfd01730bf44be87c8bb2a664d"
        },
        "date": 1780724232478,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80639,
            "range": "± 883",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 94426,
            "range": "± 457",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3076,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 858,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1614,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1010,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 104,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 108,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 102,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 199,
            "range": "± 4",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "hanyul.ryu@hanyul.xyz",
            "name": "Pignu",
            "username": "pignuante"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1e078d7196a67fd672b0f33c109fd4606dc2096f",
          "message": "Merge pull request #128 from ai-screams/release-plz-2026-06-06T05-34-28Z\n\nchore: release v0.14.0",
          "timestamp": "2026-06-06T16:41:06+09:00",
          "tree_id": "4f2e6a11bbb525288b0c8d0dbad63348a729c335",
          "url": "https://github.com/ai-screams/scoop-uv/commit/1e078d7196a67fd672b0f33c109fd4606dc2096f"
        },
        "date": 1780731896837,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 81400,
            "range": "± 1273",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 94615,
            "range": "± 727",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3115,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 805,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1580,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 994,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 97,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 87,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 85,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 21,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 191,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}