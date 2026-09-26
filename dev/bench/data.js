window.BENCHMARK_DATA = {
  "lastUpdate": 1790409324864,
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
          "id": "4877846b4363f011236a91816eb89357b0901a53",
          "message": "Merge pull request #129 from ai-screams/chore/auto-register-guards\n\ntest(cli): auto-guard man/completions for every non-hidden subcommand",
          "timestamp": "2026-06-06T16:41:55+09:00",
          "tree_id": "ce82d8cd4d7658d7ab45073a1c9ce92e96a9f791",
          "url": "https://github.com/ai-screams/scoop-uv/commit/4877846b4363f011236a91816eb89357b0901a53"
        },
        "date": 1780732119749,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 82351,
            "range": "± 910",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93823,
            "range": "± 1238",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3093,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 808,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1631,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1001,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 97,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 99,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 85,
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
          "id": "99c11a084c2f04a1fe0aa4fa8c8dadbe7651eaf4",
          "message": "Merge pull request #130 from ai-screams/fix/doctor-system-sentinel\n\nfix(doctor): treat .scoop-version: system as valid sentinel",
          "timestamp": "2026-06-06T20:04:16+09:00",
          "tree_id": "6957eacc2cf8b8a004dccf9b111d64ea5b48088c",
          "url": "https://github.com/ai-screams/scoop-uv/commit/99c11a084c2f04a1fe0aa4fa8c8dadbe7651eaf4"
        },
        "date": 1780744073589,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80760,
            "range": "± 1032",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93617,
            "range": "± 589",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3132,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 795,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1612,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 999,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 97,
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
          "id": "efa50c1147abe3b485a1682fb95d47ea8a7f55b2",
          "message": "Merge pull request #131 from ai-screams/release-plz-2026-06-06T11-04-53Z\n\nchore: release v0.14.1",
          "timestamp": "2026-06-07T08:10:55+09:00",
          "tree_id": "eeecd25d5904dd7e6375969f7afabb439fd74167",
          "url": "https://github.com/ai-screams/scoop-uv/commit/efa50c1147abe3b485a1682fb95d47ea8a7f55b2"
        },
        "date": 1780787665980,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80213,
            "range": "± 2097",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 95474,
            "range": "± 794",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3113,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 813,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1619,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1041,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 111,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 99,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 94,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 23,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 200,
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
          "id": "50d7c85efa0559d28a8e27eb54e8e38943266da3",
          "message": "Merge pull request #132 from ai-screams/dependabot/cargo/rust-dependencies-23b8550d7b\n\nchore(deps): bump which from 8.0.2 to 8.0.3 in the rust-dependencies group",
          "timestamp": "2026-06-08T10:43:17+09:00",
          "tree_id": "cc13c08507634bbfbc808576d82555496cb70a18",
          "url": "https://github.com/ai-screams/scoop-uv/commit/50d7c85efa0559d28a8e27eb54e8e38943266da3"
        },
        "date": 1780883210558,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75230,
            "range": "± 245",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88272,
            "range": "± 517",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3321,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 933,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1728,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1132,
            "range": "± 57",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 107,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 94,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 89,
            "range": "± 1",
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
            "value": 214,
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
          "id": "51aa6e4bcd338be3744c08827c9a431c57e5484c",
          "message": "Merge pull request #133 from ai-screams/dependabot/github_actions/github-actions-484570b1b1\n\nci(deps): bump codecov/codecov-action from 6 to 7 in the github-actions group",
          "timestamp": "2026-06-08T10:43:40+09:00",
          "tree_id": "ea12aff704e5f0a9cc44818e88d61f89cf2c4ac0",
          "url": "https://github.com/ai-screams/scoop-uv/commit/51aa6e4bcd338be3744c08827c9a431c57e5484c"
        },
        "date": 1780883430052,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75734,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87949,
            "range": "± 458",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3287,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 868,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1689,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1091,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 102,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 106,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 93,
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
            "value": 208,
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
          "id": "94b225c711722551550a6aa6a2f74a7c8af7829e",
          "message": "Merge pull request #134 from ai-screams/dependabot/cargo/rust-dependencies-f269572c02\n\nchore(deps): bump the rust-dependencies group with 2 updates",
          "timestamp": "2026-06-15T11:32:44+09:00",
          "tree_id": "c1e5cbf823738b33e70744f81199407751b640e3",
          "url": "https://github.com/ai-screams/scoop-uv/commit/94b225c711722551550a6aa6a2f74a7c8af7829e"
        },
        "date": 1781491029488,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 81701,
            "range": "± 4874",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 94516,
            "range": "± 1044",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3105,
            "range": "± 206",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 805,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1574,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1002,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 87,
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
          "id": "3439031ecbc33a7c622d4825726b6171e1a0d123",
          "message": "Merge pull request #135 from ai-screams/docs/refresh-0.14.1\n\ndocs: refresh documentation to 0.14.1 (code fact-check)",
          "timestamp": "2026-06-16T15:09:49+09:00",
          "tree_id": "d9b149138ac4e01cc4eb1559bb7e93f0c4503a92",
          "url": "https://github.com/ai-screams/scoop-uv/commit/3439031ecbc33a7c622d4825726b6171e1a0d123"
        },
        "date": 1781590404943,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 74118,
            "range": "± 1430",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 86965,
            "range": "± 1256",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3291,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 830,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1675,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1098,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 105,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 107,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 94,
            "range": "± 3",
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
            "value": 213,
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
          "id": "df4819e45057b585c85f6726bbac403476c80e0a",
          "message": "Merge pull request #136 from ai-screams/docs/ko-translation\n\ndocs(i18n): complete Korean documentation translation (ko.po)",
          "timestamp": "2026-06-16T16:48:49+09:00",
          "tree_id": "02f53a6c4a0a4ac143d6d8c0480de85bc7618aa5",
          "url": "https://github.com/ai-screams/scoop-uv/commit/df4819e45057b585c85f6726bbac403476c80e0a"
        },
        "date": 1781596354106,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75902,
            "range": "± 1268",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 90135,
            "range": "± 3818",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3226,
            "range": "± 79",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 834,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1679,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1101,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 104,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 93,
            "range": "± 1",
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
          "id": "0cdad77d5c09034fe9320b760285cca747ce097e",
          "message": "Merge pull request #137 from ai-screams/dependabot/cargo/rust-dependencies-ab39f97599\n\nchore(deps): bump which from 8.0.3 to 8.0.4 in the rust-dependencies group",
          "timestamp": "2026-07-02T13:06:01+09:00",
          "tree_id": "877ee56cb559a8313e0cf9a4d47398c3bb9f13c4",
          "url": "https://github.com/ai-screams/scoop-uv/commit/0cdad77d5c09034fe9320b760285cca747ce097e"
        },
        "date": 1782965375429,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 61422,
            "range": "± 281",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 71179,
            "range": "± 541",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2521,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 650,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1285,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 843,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 88,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 88,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 85,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 87,
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
            "value": 174,
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
          "id": "48ae680b61bc7888cf7a02fd0201d0155083241f",
          "message": "Merge pull request #139 from ai-screams/dependabot/github_actions/github-actions-02325a8da5\n\nci(deps): bump the github-actions group across 1 directory with 2 updates",
          "timestamp": "2026-07-02T13:06:27+09:00",
          "tree_id": "03705b60b9da58cef8a58163353174b20f19d673",
          "url": "https://github.com/ai-screams/scoop-uv/commit/48ae680b61bc7888cf7a02fd0201d0155083241f"
        },
        "date": 1782965594246,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 84394,
            "range": "± 356",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 98620,
            "range": "± 361",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3049,
            "range": "± 37",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 819,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1658,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1066,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 100,
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
            "value": 97,
            "range": "± 2",
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
            "value": 200,
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
          "id": "18603881ead091ec1746f82e4e0a0ed644c8388b",
          "message": "Merge pull request #140 from ai-screams/dependabot/cargo/rust-dependencies-91d13d154f\n\nchore(deps): bump the rust-dependencies group with 2 updates",
          "timestamp": "2026-07-10T17:22:34+09:00",
          "tree_id": "5c3a7b0df001981e7d44dc31e0a6877c11b15e07",
          "url": "https://github.com/ai-screams/scoop-uv/commit/18603881ead091ec1746f82e4e0a0ed644c8388b"
        },
        "date": 1783672014447,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80460,
            "range": "± 923",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92798,
            "range": "± 778",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3086,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 809,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1687,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1078,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 109,
            "range": "± 4",
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
            "value": 94,
            "range": "± 2",
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
            "value": 202,
            "range": "± 3",
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
          "id": "ef558b1f7991a27c3f469608630f2f7a4dacadc7",
          "message": "fix(deps): bump crossbeam-epoch 0.9.20, anyhow 1.0.103 (RUSTSEC-2026-0204) (#141)\n\nScheduled Security scan (run #368) failed after new advisories were\npublished against the committed lockfile:\n\n- RUSTSEC-2026-0204 (error): crossbeam-epoch 0.9.18 invalid pointer\n  dereference in fmt::Pointer for Atomic/Shared -> 0.9.20 (transitive\n  via rayon / rust-i18n's ignore)\n- RUSTSEC-2026-0190 (warning): anyhow 1.0.102 Error::downcast_mut()\n  unsoundness -> 1.0.103\n\nLockfile-only, MSRV 1.85 compatible. Verified: cargo audit clean,\ncargo deny --all-features check ok, cargo check --all-targets passes.",
          "timestamp": "2026-07-12T11:04:31+09:00",
          "tree_id": "6bd3c2205509298aaa5bff40cd9b55b77e63cd2a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/ef558b1f7991a27c3f469608630f2f7a4dacadc7"
        },
        "date": 1783822087362,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 83921,
            "range": "± 1255",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 99327,
            "range": "± 1354",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3075,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 824,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1647,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1074,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 103,
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
            "value": 96,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 96,
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
            "value": 200,
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
          "id": "4a5530f92b83919fbf905b3fb6042180677f38c8",
          "message": "chore: release v0.14.2 (#142)",
          "timestamp": "2026-07-12T11:25:12+09:00",
          "tree_id": "92b6117af66ed84a34d79efde44919c081d05b99",
          "url": "https://github.com/ai-screams/scoop-uv/commit/4a5530f92b83919fbf905b3fb6042180677f38c8"
        },
        "date": 1783823358601,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 82250,
            "range": "± 1029",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 95617,
            "range": "± 800",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3070,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 870,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1664,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1067,
            "range": "± 2",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 99,
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
            "value": 193,
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
          "id": "ff9e0f80c89cfbc56585c25674471ea4708e9f2c",
          "message": "Merge pull request #143 from ai-screams/feat/scuv-rename\n\nfeat(rename)!: rename CLI command scoop -> scuv (v0.15.0)",
          "timestamp": "2026-07-13T18:03:15+09:00",
          "tree_id": "1c4a17497e8cd786c2998d5d0e58370453b056e5",
          "url": "https://github.com/ai-screams/scoop-uv/commit/ff9e0f80c89cfbc56585c25674471ea4708e9f2c"
        },
        "date": 1783933613529,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 81728,
            "range": "± 980",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 96964,
            "range": "± 9090",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3077,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 861,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1673,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1061,
            "range": "± 3",
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
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 87,
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
          "id": "3018d537d122a39b577cd968a755d4fe6af13a64",
          "message": "Merge pull request #144 from ai-screams/release-plz-2026-07-13T09-04-13Z\n\nchore: release v0.15.0",
          "timestamp": "2026-07-13T19:09:11+09:00",
          "tree_id": "f82ee325d2a8c5a4cc051ff861662fbf20d126e2",
          "url": "https://github.com/ai-screams/scoop-uv/commit/3018d537d122a39b577cd968a755d4fe6af13a64"
        },
        "date": 1783937578274,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 78783,
            "range": "± 270",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 91205,
            "range": "± 595",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3047,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 784,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1691,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1078,
            "range": "± 13",
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
            "range": "± 1",
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
            "value": 191,
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
          "id": "b58a13145a17883cf490ab47d86157f052d86d11",
          "message": "Merge pull request #145 from ai-screams/fix/upgrade-docs-and-fuzz-target\n\ndocs: upgrading-from-scoop guide + fuzz gnu-target fix",
          "timestamp": "2026-07-14T22:25:51+09:00",
          "tree_id": "4f3f2a71fd83d6db62ab07c5c68f86f1b0aa7994",
          "url": "https://github.com/ai-screams/scoop-uv/commit/b58a13145a17883cf490ab47d86157f052d86d11"
        },
        "date": 1784035767016,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75465,
            "range": "± 2164",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88639,
            "range": "± 1043",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3493,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 838,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1662,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1074,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 102,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 103,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 97,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 90,
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
            "value": 210,
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
          "id": "7d8cfe71fc4cb772cf240dab9e5fd4d5663f61d4",
          "message": "Merge pull request #146 from ai-screams/chore/0.15.1-backlog\n\nfix: v0.15.1 follow-up — panic fix, double-warning, test hardening",
          "timestamp": "2026-07-15T08:53:40+09:00",
          "tree_id": "91be27fb624d8cb0785b38518cb00f86084f5556",
          "url": "https://github.com/ai-screams/scoop-uv/commit/7d8cfe71fc4cb772cf240dab9e5fd4d5663f61d4"
        },
        "date": 1784073435220,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 62604,
            "range": "± 1490",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 76959,
            "range": "± 2287",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2485,
            "range": "± 88",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 663,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 574,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 398,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 58,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 61,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 74,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 66,
            "range": "± 2",
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
            "value": 145,
            "range": "± 6",
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
          "id": "f659098ee779bca7a700f1f036a30cfaaa639736",
          "message": "Merge pull request #147 from ai-screams/release-plz-2026-07-14T23-54-19Z\n\nchore: release v0.15.1",
          "timestamp": "2026-07-15T14:35:58+09:00",
          "tree_id": "dafcf56201b6711b33642fa1f4fc31ce7826f3dd",
          "url": "https://github.com/ai-screams/scoop-uv/commit/f659098ee779bca7a700f1f036a30cfaaa639736"
        },
        "date": 1784093979022,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80803,
            "range": "± 720",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93456,
            "range": "± 1687",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3067,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 799,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1689,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1065,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 98,
            "range": "± 4",
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
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 87,
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
          "id": "80beb4e0c157c3a70ce31427d369ed16a2fd0362",
          "message": "Merge pull request #149 from ai-screams/refactor/split-doctor\n\nrefactor(doctor): split doctor.rs into a focused doctor/ module",
          "timestamp": "2026-07-19T21:30:43+09:00",
          "tree_id": "14bf06b2c863d20abe686a7a68de197e84e8d67d",
          "url": "https://github.com/ai-screams/scoop-uv/commit/80beb4e0c157c3a70ce31427d369ed16a2fd0362"
        },
        "date": 1784464503756,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75570,
            "range": "± 1963",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87397,
            "range": "± 395",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3148,
            "range": "± 100",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 807,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1888,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1268,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 103,
            "range": "± 0",
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
            "value": 93,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 89,
            "range": "± 1",
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
          "id": "c034aa3f1f856083e03baf46bbb20eb2e2efc500",
          "message": "Merge pull request #150 from ai-screams/chore/bump-msrv-1.88\n\nfix(msrv): bump to 1.88 — ecosystem adopted let-chains",
          "timestamp": "2026-07-19T23:16:02+09:00",
          "tree_id": "84bc93253ba02917335f3b58680680333d8c7049",
          "url": "https://github.com/ai-screams/scoop-uv/commit/c034aa3f1f856083e03baf46bbb20eb2e2efc500"
        },
        "date": 1784470824321,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 73679,
            "range": "± 1310",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88731,
            "range": "± 347",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3122,
            "range": "± 65",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 815,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1931,
            "range": "± 21",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1273,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 98,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 99,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 90,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 88,
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
            "value": 204,
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
          "id": "c217de2f6cf24e04d1ea64f5c4a44a882f8258b8",
          "message": "Merge pull request #148 from ai-screams/release-plz-2026-07-15T05-36-39Z\n\nchore: release v0.15.2",
          "timestamp": "2026-07-19T23:45:15+09:00",
          "tree_id": "d755a933e675cbcc935eae9d0d3b2cadb1adce59",
          "url": "https://github.com/ai-screams/scoop-uv/commit/c217de2f6cf24e04d1ea64f5c4a44a882f8258b8"
        },
        "date": 1784472555842,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 61940,
            "range": "± 481",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 72499,
            "range": "± 336",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2442,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 609,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1460,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 981,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 76,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 60,
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
            "value": 157,
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
          "id": "67324d6ccec124d2edd879acc47155b459e7bf88",
          "message": "Merge pull request #152 from ai-screams/dependabot/cargo/rust-dependencies-159dc18219\n\nchore(deps): bump criterion from 0.7.0 to 0.8.2 in the rust-dependencies group across 1 directory",
          "timestamp": "2026-07-20T00:17:23+09:00",
          "tree_id": "ec4f0ecf1e5356fba0dff72df5e0223ca25b2387",
          "url": "https://github.com/ai-screams/scoop-uv/commit/67324d6ccec124d2edd879acc47155b459e7bf88"
        },
        "date": 1784474458494,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 84779,
            "range": "± 1233",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92435,
            "range": "± 497",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3074,
            "range": "± 38",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 825,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1705,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1072,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 94,
            "range": "± 0",
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
            "value": 78,
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
            "value": 187,
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
          "id": "1d805cb50312e7a3e61005e7b676b30ebb5257fd",
          "message": "Merge pull request #153 from ai-screams/docs/pages-url-custom-domain\n\ndocs: point Pages URLs at ai-scream.ai custom domain",
          "timestamp": "2026-07-20T23:32:30+09:00",
          "tree_id": "7019a0f2ce16aebfc98b96b6d3db8a30332193c2",
          "url": "https://github.com/ai-screams/scoop-uv/commit/1d805cb50312e7a3e61005e7b676b30ebb5257fd"
        },
        "date": 1784558165018,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80146,
            "range": "± 446",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92700,
            "range": "± 1110",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3046,
            "range": "± 27",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 852,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1664,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1081,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 89,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 78,
            "range": "± 2",
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
            "value": 186,
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
          "id": "ab04234c0841f376e2be93ec4c8409db6f880347",
          "message": "Merge pull request #158 from ai-screams/dependabot/cargo/rust-dependencies-a3cc385a98\n\nchore(deps): bump the rust-dependencies group across 1 directory with 8 updates",
          "timestamp": "2026-09-04T13:17:44+09:00",
          "tree_id": "cb835ef04e0d8bf8efea333b54660ab62fd8e7b8",
          "url": "https://github.com/ai-screams/scoop-uv/commit/ab04234c0841f376e2be93ec4c8409db6f880347"
        },
        "date": 1788495728296,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 82026,
            "range": "± 1006",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 94027,
            "range": "± 687",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3172,
            "range": "± 41",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 804,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1682,
            "range": "± 34",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1123,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 95,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 78,
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
            "value": 188,
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
          "id": "70710c2ae93875504304716329a1f49a0b11c4ca",
          "message": "Merge pull request #159 from ai-screams/docs/i18n-guide-touchpoints\n\ndocs(i18n): document the locale lists the translation guide was missing",
          "timestamp": "2026-09-05T10:04:13+09:00",
          "tree_id": "b235b121af6454d642ff44d6141aa407f5384f05",
          "url": "https://github.com/ai-screams/scoop-uv/commit/70710c2ae93875504304716329a1f49a0b11c4ca"
        },
        "date": 1788570467841,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 76059,
            "range": "± 938",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87273,
            "range": "± 1701",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3223,
            "range": "± 191",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 779,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 1900,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1328,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 100,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 92,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 84,
            "range": "± 1",
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
            "value": 203,
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
          "id": "5d2bb6530480f5a31067abc6aa30cb06af773471",
          "message": "Merge pull request #160 from ai-screams/fix/ko-po-msgfmt\n\nfix(docs): repair the malformed ko.po entry and gate .po files in CI",
          "timestamp": "2026-09-05T12:36:05+09:00",
          "tree_id": "992d904f534f2cd066abef714bc530b6f7c4bed1",
          "url": "https://github.com/ai-screams/scoop-uv/commit/5d2bb6530480f5a31067abc6aa30cb06af773471"
        },
        "date": 1788579602173,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 72393,
            "range": "± 1127",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88327,
            "range": "± 1227",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2781,
            "range": "± 52",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 729,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 668,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 488,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 66,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 71,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 86,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 68,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 20,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 168,
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
          "id": "28328b3f7d1a4282b1a76e5baccbcd311b820c3d",
          "message": "Merge pull request #161 from ai-screams/docs/refresh-stale-references\n\ndocs: finish the scuv rename in examples/ and guard reference drift in CI",
          "timestamp": "2026-09-05T13:51:38+09:00",
          "tree_id": "8cc1091994145c551a6d1458ad15123510b760ad",
          "url": "https://github.com/ai-screams/scoop-uv/commit/28328b3f7d1a4282b1a76e5baccbcd311b820c3d"
        },
        "date": 1788584120130,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 72735,
            "range": "± 618",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87215,
            "range": "± 330",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2762,
            "range": "± 32",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 718,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_hit",
            "value": 718,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 530,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 63,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 69,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 82,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 164,
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
          "id": "3fd7fa28e6548c38d2472433dd8d7491ef00a2d7",
          "message": "Merge pull request #162 from ai-screams/ci/split-io-benchmarks\n\nci(bench): track the filesystem benchmarks instead of gating on them",
          "timestamp": "2026-09-05T14:14:09+09:00",
          "tree_id": "7de380b4a8223e6c5959d83ddde6ab94bd2f2511",
          "url": "https://github.com/ai-screams/scoop-uv/commit/3fd7fa28e6548c38d2472433dd8d7491ef00a2d7"
        },
        "date": 1788585483739,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 82004,
            "range": "± 1043",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 97995,
            "range": "± 1584",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3131,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 812,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 94,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 87,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 186,
            "range": "± 6",
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
          "id": "70eb2397ca02ef6499d2ff0217d33e2b11304769",
          "message": "Merge pull request #164 from ai-screams/ci/unblock-quality-gates\n\nci: let the weekly mutation run finish, and serialise release-plz",
          "timestamp": "2026-09-06T02:05:34+09:00",
          "tree_id": "45076640d780d087a060da549715cc893b4092ac",
          "url": "https://github.com/ai-screams/scoop-uv/commit/70eb2397ca02ef6499d2ff0217d33e2b11304769"
        },
        "date": 1788628170263,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80407,
            "range": "± 447",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 91848,
            "range": "± 685",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3121,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 813,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 95,
            "range": "± 0",
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
            "value": 78,
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
            "value": 186,
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
          "id": "ac604c2a13ebfde2919fbf9fcb1ccf0826f7c80f",
          "message": "Merge pull request #166 from ai-screams/ci/cache-and-action-pins\n\nci: give each workflow its real cache key, and pin the two loose actions",
          "timestamp": "2026-09-06T02:05:42+09:00",
          "tree_id": "90e016f0116fe04b590b0a19337a36f1509bbb6c",
          "url": "https://github.com/ai-screams/scoop-uv/commit/ac604c2a13ebfde2919fbf9fcb1ccf0826f7c80f"
        },
        "date": 1788628419753,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 76460,
            "range": "± 492",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87793,
            "range": "± 701",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3267,
            "range": "± 82",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 804,
            "range": "± 31",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
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
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 82,
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
            "value": 203,
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
          "id": "a0ced1aff7d3585e56bc8bb4b2d66a7a08f0a43f",
          "message": "Merge pull request #163 from ai-screams/docs/ci-cd-design\n\ndocs: document the CI/CD design",
          "timestamp": "2026-09-06T02:14:46+09:00",
          "tree_id": "967b192d627c8457daaf76345805d3b2e412ccbb",
          "url": "https://github.com/ai-screams/scoop-uv/commit/a0ced1aff7d3585e56bc8bb4b2d66a7a08f0a43f"
        },
        "date": 1788628725645,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 81464,
            "range": "± 809",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92502,
            "range": "± 616",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3108,
            "range": "± 35",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 825,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 88,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 187,
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
          "id": "8140def69dbe22652ff4d17140f348f86bac2067",
          "message": "Merge pull request #167 from ai-screams/fix/uv-min-version\n\nfix(uv): correct MIN_VERSION to 0.5.19 and verify the floor in CI",
          "timestamp": "2026-09-06T10:46:41+09:00",
          "tree_id": "5f1ba6ce9353b309197900d31e358aa2307c27cc",
          "url": "https://github.com/ai-screams/scoop-uv/commit/8140def69dbe22652ff4d17140f348f86bac2067"
        },
        "date": 1788659447861,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 69146,
            "range": "± 557",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 86671,
            "range": "± 2509",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2647,
            "range": "± 45",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 734,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 62,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 81,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 66,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 19,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 162,
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
          "id": "1186a36f70269f6d6c3536c567c5bf7cc8481a39",
          "message": "Merge pull request #168 from ai-screams/fix/migrate-batch-name-collision\n\nfix(migrate): reject duplicate names within one batch; label env-var source",
          "timestamp": "2026-09-06T15:08:42+09:00",
          "tree_id": "a5b84b04061fbc772fc287e8350a321adf08ecb8",
          "url": "https://github.com/ai-screams/scoop-uv/commit/1186a36f70269f6d6c3536c567c5bf7cc8481a39"
        },
        "date": 1788675162259,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 82224,
            "range": "± 1450",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 94369,
            "range": "± 784",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3100,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 766,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 77,
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
            "value": 185,
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
          "id": "8759a7c3514daab45a573ed568e88282d0a05a6e",
          "message": "Merge pull request #169 from ai-screams/release-plz-2026-09-06T01-47-21Z\n\nchore: release v0.15.3",
          "timestamp": "2026-09-06T19:01:20+09:00",
          "tree_id": "5d2d930aafd1b7945329b653d9e2aa4dc54fc939",
          "url": "https://github.com/ai-screams/scoop-uv/commit/8759a7c3514daab45a573ed568e88282d0a05a6e"
        },
        "date": 1788689149864,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80193,
            "range": "± 473",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93169,
            "range": "± 3094",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3177,
            "range": "± 144",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 773,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 93,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 77,
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
            "value": 186,
            "range": "± 7",
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
          "id": "b9437c8dacc1dca6198b846917ad8d4e9c987658",
          "message": "Merge pull request #170 from ai-screams/fix/release-plz-toolchain\n\nci(release): run release-plz under stable, not the MSRV pin",
          "timestamp": "2026-09-06T19:41:21+09:00",
          "tree_id": "6f4365daa4a54007b2374775644253963ff9f9af",
          "url": "https://github.com/ai-screams/scoop-uv/commit/b9437c8dacc1dca6198b846917ad8d4e9c987658"
        },
        "date": 1788691521090,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75127,
            "range": "± 782",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87162,
            "range": "± 777",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3279,
            "range": "± 33",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 777,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 96,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 100,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 81,
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
            "value": 205,
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
          "id": "18a9ecee6d56bb4448d0888bf05fd3bbfa30c7ee",
          "message": "Merge pull request #171 from ai-screams/test/kill-migrate-mutants\n\ntest(migrate): close the mutation gaps the weekly run had been finding",
          "timestamp": "2026-09-10T16:31:01+09:00",
          "tree_id": "2ceb8c522dcc2ad96b6ff6537bb9ce8fe4516a44",
          "url": "https://github.com/ai-screams/scoop-uv/commit/18a9ecee6d56bb4448d0888bf05fd3bbfa30c7ee"
        },
        "date": 1789025772295,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75426,
            "range": "± 594",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87044,
            "range": "± 417",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3280,
            "range": "± 84",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 796,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 96,
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
            "value": 89,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 83,
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
            "value": 203,
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
          "id": "88befe7cb72a9106c48b09cba1188efd40c8ca51",
          "message": "Merge pull request #172 from ai-screams/docs/refresh-drift-2026-09\n\ndocs: refresh every doc surface against the code",
          "timestamp": "2026-09-10T23:29:28+09:00",
          "tree_id": "87ed7c7245a16ca9249abce3b33d566025a8d2cf",
          "url": "https://github.com/ai-screams/scoop-uv/commit/88befe7cb72a9106c48b09cba1188efd40c8ca51"
        },
        "date": 1789050813700,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 76234,
            "range": "± 1163",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87886,
            "range": "± 823",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3267,
            "range": "± 18",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 789,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 98,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 81,
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
            "value": 202,
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
          "id": "dedceef2ee04e31f45094eee47512a4e3e12fbb9",
          "message": "Merge pull request #175 from ai-screams/chore/bump-msrv-1.89\n\nfix(msrv): [#173] bump to 1.89 so the rust-dependencies group can land",
          "timestamp": "2026-09-16T00:03:54+09:00",
          "tree_id": "185298adde404931978031da9127ab073ce0f187",
          "url": "https://github.com/ai-screams/scoop-uv/commit/dedceef2ee04e31f45094eee47512a4e3e12fbb9"
        },
        "date": 1789484914449,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80005,
            "range": "± 908",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92389,
            "range": "± 660",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3083,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 773,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 98,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 103,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 96,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 86,
            "range": "± 3",
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
            "value": 195,
            "range": "± 5",
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
          "id": "51c5c1556099f64bcf896e6ad04782f166d60ddd",
          "message": "Merge pull request #177 from ai-screams/fix/release-pr-version-samples\n\nfix(release): [#176] sync version samples into the release PR automatically",
          "timestamp": "2026-09-16T13:51:59+09:00",
          "tree_id": "53dd23db65d69db4c0e30e5d7601094d65edb5a8",
          "url": "https://github.com/ai-screams/scoop-uv/commit/51c5c1556099f64bcf896e6ad04782f166d60ddd"
        },
        "date": 1789534553472,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80300,
            "range": "± 1206",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92806,
            "range": "± 1291",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3080,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 778,
            "range": "± 7",
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
            "value": 103,
            "range": "± 0",
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
            "value": 87,
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
            "value": 195,
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
          "id": "61402c826067419b3669b80347de6e1d9c300b37",
          "message": "Merge pull request #176 from ai-screams/release-plz-2026-09-15T15-04-23Z\n\nchore: release v0.15.4",
          "timestamp": "2026-09-16T14:00:40+09:00",
          "tree_id": "bafb3cc6aee5fcf6c4d75a9a9c5aff2a1bc39675",
          "url": "https://github.com/ai-screams/scoop-uv/commit/61402c826067419b3669b80347de6e1d9c300b37"
        },
        "date": 1789535101384,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 74125,
            "range": "± 774",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88183,
            "range": "± 1033",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3191,
            "range": "± 95",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 850,
            "range": "± 76",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 98,
            "range": "± 4",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 90,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 81,
            "range": "± 4",
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
            "value": 203,
            "range": "± 5",
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
          "id": "883f1b302492cf409afb3eb82ef14bd3bf88a039",
          "message": "Merge pull request #174 from ai-screams/dependabot/github_actions/github-actions-03db81fe05\n\nci(deps): bump astral-sh/setup-uv from 10.0.1 to 10.1.0 in the github-actions group",
          "timestamp": "2026-09-16T14:09:17+09:00",
          "tree_id": "c20e4c35308bbea4a26e74d73d7445314c5a278a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/883f1b302492cf409afb3eb82ef14bd3bf88a039"
        },
        "date": 1789535593900,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75262,
            "range": "± 1566",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88550,
            "range": "± 802",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3238,
            "range": "± 19",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 845,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 81,
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
            "value": 203,
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
          "id": "50b29942fc9695d2436e21354be394df099a7f9e",
          "message": "Merge pull request #178 from ai-screams/fix/ko-po-and-dependabot-ignore\n\nfix(docs): regenerate ko.po for the 1.89 docs and gate serial_test 4.x",
          "timestamp": "2026-09-16T16:06:34+09:00",
          "tree_id": "2a8fde0e08d063db1b6bf4ef6b3e7e839042debf",
          "url": "https://github.com/ai-screams/scoop-uv/commit/50b29942fc9695d2436e21354be394df099a7f9e"
        },
        "date": 1789542630945,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 62318,
            "range": "± 1032",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 78228,
            "range": "± 1887",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2599,
            "range": "± 69",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 655,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 59,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 62,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 75,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 59,
            "range": "± 1",
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
            "value": 146,
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
          "id": "d5fe6122014d66370d63c1a58391d1f404492ac7",
          "message": "Merge pull request #179 from ai-screams/docs/claude-md-session-learnings\n\ndocs: sync the reference docs and CLAUDE.md to v0.15.4",
          "timestamp": "2026-09-16T16:26:40+09:00",
          "tree_id": "4b4f7b91f9490ed2a14b4a43e697e224cf8c123c",
          "url": "https://github.com/ai-screams/scoop-uv/commit/d5fe6122014d66370d63c1a58391d1f404492ac7"
        },
        "date": 1789543843129,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75343,
            "range": "± 1134",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87307,
            "range": "± 3605",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3192,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 770,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 96,
            "range": "± 1",
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
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 84,
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
            "value": 204,
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
          "id": "eb93345308cbc5d616964c29b233de3d379dfe0b",
          "message": "Merge pull request #180 from ai-screams/fix/docs-guard-on-pr\n\nci(docs): run the docs guards on PRs and drop the unmaintained serde_yaml",
          "timestamp": "2026-09-16T17:54:39+09:00",
          "tree_id": "5bf3a809c331208beea5f90f790dd99ec210bc28",
          "url": "https://github.com/ai-screams/scoop-uv/commit/eb93345308cbc5d616964c29b233de3d379dfe0b"
        },
        "date": 1789549115737,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 74413,
            "range": "± 718",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88059,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3243,
            "range": "± 48",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 854,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 98,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 81,
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
            "value": 204,
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
          "id": "fe6af580c4ff9323f8bb0b74ca9e0f339cacef99",
          "message": "Merge pull request #181 from ai-screams/fix/api-md-date-stamp\n\ndocs(api): drop the ambiguous Last Updated stamp from the API reference",
          "timestamp": "2026-09-16T22:43:01+09:00",
          "tree_id": "583f67b9a4a6e1993621f3da02ced8c1350aeb03",
          "url": "https://github.com/ai-screams/scoop-uv/commit/fe6af580c4ff9323f8bb0b74ca9e0f339cacef99"
        },
        "date": 1789566424852,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75445,
            "range": "± 439",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 88462,
            "range": "± 544",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3251,
            "range": "± 44",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 838,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 96,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 99,
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
            "value": 80,
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
            "value": 204,
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
          "id": "04d3d72189f4264e1d27fe49bc1e6aa1c04ca935",
          "message": "Merge pull request #182 from ai-screams/fix/saphyr-and-docs-check\n\nrefactor(ci): split PR docs checks from deploy and use the already-locked serde-saphyr",
          "timestamp": "2026-09-16T22:43:41+09:00",
          "tree_id": "949df8abb3034e4ff97c03545424db169589e3c7",
          "url": "https://github.com/ai-screams/scoop-uv/commit/04d3d72189f4264e1d27fe49bc1e6aa1c04ca935"
        },
        "date": 1789566674317,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80249,
            "range": "± 1055",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93053,
            "range": "± 561",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3191,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 781,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 94,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 80,
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
            "value": 186,
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
          "id": "b3bafed1c5a72531b3dc5a0fcb93e06ce5693031",
          "message": "Merge pull request #184 from ai-screams/ci/codecov-thresholds\n\nci(coverage): state a coverage target instead of reporting into the void",
          "timestamp": "2026-09-17T00:00:20+09:00",
          "tree_id": "ef09f19218223d17a2727df3befa650de77c9da4",
          "url": "https://github.com/ai-screams/scoop-uv/commit/b3bafed1c5a72531b3dc5a0fcb93e06ce5693031"
        },
        "date": 1789571064599,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 74610,
            "range": "± 1458",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87103,
            "range": "± 1010",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3322,
            "range": "± 49",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 784,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
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
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 83,
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
            "value": 204,
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
          "id": "9a9241dae093ae0e351d27a0c7f21151c86b92c3",
          "message": "Merge pull request #185 from ai-screams/ci/stable-msrv-job-name\n\nci(msrv): drop the version from the MSRV job name",
          "timestamp": "2026-09-17T00:27:23+09:00",
          "tree_id": "a310dbc41370bfb8f3cfda96689f820ea9ea4f87",
          "url": "https://github.com/ai-screams/scoop-uv/commit/9a9241dae093ae0e351d27a0c7f21151c86b92c3"
        },
        "date": 1789572654676,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 44637,
            "range": "± 2609",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 55049,
            "range": "± 3127",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 1808,
            "range": "± 122",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 523,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 52,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 53,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 49,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 43,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/reserved_reject",
            "value": 14,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 124,
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
          "id": "cd592c4fd7dcd73f9259f68ef083b712616d8c91",
          "message": "Merge pull request #186 from ai-screams/fix/release-sync-ko-po\n\nfix(release): keep ko.po in sync when the release PR rewrites version samples",
          "timestamp": "2026-09-17T01:24:48+09:00",
          "tree_id": "49a7b7200c2a38e7118dac38c1fc8d20a3249335",
          "url": "https://github.com/ai-screams/scoop-uv/commit/cd592c4fd7dcd73f9259f68ef083b712616d8c91"
        },
        "date": 1789576107474,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 61159,
            "range": "± 617",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 69978,
            "range": "± 745",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 2620,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 607,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 73,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 77,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 67,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 60,
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
            "value": 157,
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
          "id": "37ab23726ae47026e9ab66c778dc85d64d0c108f",
          "message": "Merge pull request #187 from ai-screams/ci/bench-threshold-from-data\n\nci(bench): set the CPU benchmark threshold from measured variance",
          "timestamp": "2026-09-17T01:36:21+09:00",
          "tree_id": "13f36edb919da7444a771fc9b553cef6268db6cd",
          "url": "https://github.com/ai-screams/scoop-uv/commit/37ab23726ae47026e9ab66c778dc85d64d0c108f"
        },
        "date": 1789576820885,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 84086,
            "range": "± 1351",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 96537,
            "range": "± 651",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3238,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 759,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 92,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 87,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 186,
            "range": "± 3",
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
          "id": "8e427f71677c38c7bdc65a6343a2a4cfc2ae9bdf",
          "message": "Merge pull request #183 from ai-screams/release-plz-2026-09-16T13-44-01Z\n\nchore: release v0.15.5",
          "timestamp": "2026-09-17T01:45:18+09:00",
          "tree_id": "675ea5938c914da8f1f110e168ddf4334d9146b3",
          "url": "https://github.com/ai-screams/scoop-uv/commit/8e427f71677c38c7bdc65a6343a2a4cfc2ae9bdf"
        },
        "date": 1789577357526,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 80182,
            "range": "± 539",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 91990,
            "range": "± 738",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3149,
            "range": "± 14",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 758,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 93,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 187,
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
          "id": "7a305a8a523266e0a8320d0c04f69cba19e26f64",
          "message": "Merge pull request #189 from ai-screams/ci/coverage-floor\n\nci(coverage): enforce a floor locally, since codecov/project never posts",
          "timestamp": "2026-09-17T11:29:08+09:00",
          "tree_id": "eac2070c27225a47d9d90f81005aa4fbca2df73a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/7a305a8a523266e0a8320d0c04f69cba19e26f64"
        },
        "date": 1789612379905,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 79624,
            "range": "± 964",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93333,
            "range": "± 1076",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3122,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 759,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 91,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 88,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 186,
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
          "id": "808369b218a367f75f3e79a72638f1bae294d14b",
          "message": "docs(theme): add canonical/hreflang links and an Ai-Scream home link (#193)\n\n* docs(theme): add canonical/hreflang links and an Ai-Scream home link\n\nThe docs live at https://ai-scream.ai/scoop-uv/ next to the Ai-Scream\nhome page, which now lists them in its sitemap. This makes the docs\nside say so too:\n\n- per-page <link rel=\"canonical\"> on https://ai-scream.ai, with\n  /x/index.html folded into /x/, plus hreflang en/ko/x-default pairing\n  each English page with its Korean twin. Set from the head script\n  because mdBook only exposes the source .md path to the theme\n- a \"by Ai-Scream\" link (to /ko/ on Korean pages) in the locale\n  switcher\n- the switcher now sits in the menu bar's right-button row instead of\n  floating over it: the fixed overlay covered the print, git and edit\n  buttons (already true before this change; the longer label made it\n  worse). Links take the theme's icon colors; below 620px only the\n  locale links remain\n- the locale-path logic is shared by the switcher and the SEO links\n\nCloses #192\n\n* docs(theme): keep 404 and print pages out of canonical/hreflang clusters\n\nGitHub Pages serves 404.html for any missing URL, so deriving the links\nfrom location.pathname declared /scoop-uv/missing canonical and\nadvertised /scoop-uv/ko/missing as its Korean twin; a direct\n/404.html request (HTTP 200) got the same. print.html, which mdBook\nalready marks noindex, also got the links.\n\n- the template context's path (404.md / print.md) sets skipSeoLinks,\n  so those pages carry no canonical or hreflang\n- 404.html gets a static <meta name=\"robots\" content=\"noindex\">\n\nChecked in Chrome against a Pages-like server: missing URLs, /404.html\nand /print.html have 0 canonical/hreflang links and noindex; content\npages still get 1 canonical + 3 hreflang. Forcing skipSeoLinks=false\nbrings the links back on the 404 responses.",
          "timestamp": "2026-09-26T00:59:17+09:00",
          "tree_id": "8bbf6513365a66044d096690738f9b416a222dad",
          "url": "https://github.com/ai-screams/scoop-uv/commit/808369b218a367f75f3e79a72638f1bae294d14b"
        },
        "date": 1790352193340,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 79962,
            "range": "± 1023",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93815,
            "range": "± 853",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3083,
            "range": "± 16",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 758,
            "range": "± 3",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 91,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 88,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 186,
            "range": "± 0",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jsmomo0305@gmail.com",
            "name": "sayam-1",
            "username": "sayam-1"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a62dd6e316da233fe7e8d1f9c1326cba3b898378",
          "message": "feat(i18n): add Spanish translation (#190)\n\nAdds the `es` locale: every key in locales/app.yml, SUPPORTED_LANGS,\nthe i18n completeness gate, and the `scuv lang` completion lists in all\nfour shells.\n\nCo-authored-by: pignuante <hanyul.ryu@hanyul.xyz>",
          "timestamp": "2026-09-26T03:56:26+09:00",
          "tree_id": "f4a3726ff6059c62081ec4a08478d68c922c514e",
          "url": "https://github.com/ai-screams/scoop-uv/commit/a62dd6e316da233fe7e8d1f9c1326cba3b898378"
        },
        "date": 1790362829916,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 79202,
            "range": "± 507",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92167,
            "range": "± 1769",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3545,
            "range": "± 116",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 890,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 103,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 107,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 96,
            "range": "± 1",
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
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/max_length",
            "value": 218,
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
          "id": "c1fc3e917acf73edfc709d6dca1a68de6023b3b5",
          "message": "refactor(compat)!: drop the scoop-era legacy fallbacks\n\nThe 0.15.0 rename kept the scoop-era names readable behind one-shot\nwarnings that promised removal in 0.16.0. This is that removal: the\nSCOOP_* environment variables, ~/.scoop, .scoop-version and .scoop.toml\nare no longer read, the bash/zsh/fish `scoop` forwarder is gone, and\n`scuv shell` exports SCUV_VERSION only. The doctor `legacy` check stays\nas a warn-only diagnostic so an incomplete upgrade is not silent.\n.scoop-metadata.json and scoop_export_version are unchanged (on-disk\nformats).\n\nBREAKING CHANGE: SCOOP_HOME, SCOOP_VERSION, SCOOP_LANG,\nSCOOP_RESOLVE_MAX_DEPTH and SCOOP_NO_AUTO are ignored, as are ~/.scoop,\n.scoop-version and .scoop.toml. Rename them to the SCUV_* / .scuv names\nand run `mv ~/.scoop ~/.scuv`. SCUV_SUPPRESS_DEPRECATION is no longer\nrecognised.",
          "timestamp": "2026-09-26T09:47:37+09:00",
          "tree_id": "e44f9e574d14fe3a3575fdef690b9c772909030e",
          "url": "https://github.com/ai-screams/scoop-uv/commit/c1fc3e917acf73edfc709d6dca1a68de6023b3b5"
        },
        "date": 1790383899878,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 75105,
            "range": "± 903",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 87536,
            "range": "± 705",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3225,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 790,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
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
            "value": 90,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 83,
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
            "value": 203,
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
          "id": "96aca282f82541cb55e56882eed7e41332d38e1f",
          "message": "Merge pull request #194 from ai-screams/release-plz-2026-09-25T18-56-49Z\n\nchore: release v0.16.0",
          "timestamp": "2026-09-26T09:59:49+09:00",
          "tree_id": "0b4e84f13dc1338c9dffb5017ff9710ef08a39de",
          "url": "https://github.com/ai-screams/scoop-uv/commit/96aca282f82541cb55e56882eed7e41332d38e1f"
        },
        "date": 1790384650836,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 82514,
            "range": "± 432",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 95083,
            "range": "± 605",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3084,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 756,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 95,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 97,
            "range": "± 2",
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
            "value": 78,
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
            "value": 188,
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
          "id": "12817f5f64c22b18118c63172aef98e2189a8032",
          "message": "docs(i18n): fix the add-a-locale checklist and guard the language lists\n\nFixes the add-a-locale guidance that skipped the bash and PowerShell completion lists (#190), lists Spanish everywhere the supported languages are enumerated, pins each shell's completion list to SUPPORTED_LANGS with a test, guards the doc lists in check-doc-references.py, transcribes the lang command examples from the binary, brings ci-cd.md up to date with docs-check.yml, and records the merge and release conventions in CLAUDE.md.",
          "timestamp": "2026-09-26T11:13:43+09:00",
          "tree_id": "0a3662872ad0836d6b47a1676397124516bbcce5",
          "url": "https://github.com/ai-screams/scoop-uv/commit/12817f5f64c22b18118c63172aef98e2189a8032"
        },
        "date": 1790389061889,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 81193,
            "range": "± 1354",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 93314,
            "range": "± 540",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3064,
            "range": "± 23",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 775,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 92,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 95,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 90,
            "range": "± 1",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 80,
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
            "value": 186,
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
          "id": "df7ad0a5c6798541102f1436d5cefa33c2a77628",
          "message": "ci(cache): write caches only from main and tolerate export failures\n\nThe Actions cache reached 10.49 GB against the 10 GB allowance and LRU eviction failed a green Docker build during a BuildKit export (error writing layer blob: not_found). rust-cache steps now save only from main, BuildKit cache-to is emitted only off pull_request with ignore-error=true, and cache-cleanup.yml deletes a PR's caches when it closes.",
          "timestamp": "2026-09-26T16:51:33+09:00",
          "tree_id": "2eb9867cd0abf499e0a81c5f7169cf81a9914844",
          "url": "https://github.com/ai-screams/scoop-uv/commit/df7ad0a5c6798541102f1436d5cefa33c2a77628"
        },
        "date": 1790409322561,
        "tool": "cargo",
        "benches": [
          {
            "name": "clap_parse_create",
            "value": 78903,
            "range": "± 1590",
            "unit": "ns/iter"
          },
          {
            "name": "clap_parse_migrate_all",
            "value": 92450,
            "range": "± 1129",
            "unit": "ns/iter"
          },
          {
            "name": "toml_parse_scoop_manifest",
            "value": 3093,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_uv_python_list",
            "value": 758,
            "range": "± 2",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/typical",
            "value": 91,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/hyphenated",
            "value": 93,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/digit_start_reject",
            "value": 89,
            "range": "± 0",
            "unit": "ns/iter"
          },
          {
            "name": "is_valid_env_name/version_like_reject",
            "value": 79,
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
            "value": 186,
            "range": "± 1",
            "unit": "ns/iter"
          }
        ]
      }
    ],
    "scoop-uv filesystem benchmarks": [
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
          "id": "3fd7fa28e6548c38d2472433dd8d7491ef00a2d7",
          "message": "Merge pull request #162 from ai-screams/ci/split-io-benchmarks\n\nci(bench): track the filesystem benchmarks instead of gating on them",
          "timestamp": "2026-09-05T14:14:09+09:00",
          "tree_id": "7de380b4a8223e6c5959d83ddde6ab94bd2f2511",
          "url": "https://github.com/ai-screams/scoop-uv/commit/3fd7fa28e6548c38d2472433dd8d7491ef00a2d7"
        },
        "date": 1788585485804,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1696,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1114,
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
          "id": "70eb2397ca02ef6499d2ff0217d33e2b11304769",
          "message": "Merge pull request #164 from ai-screams/ci/unblock-quality-gates\n\nci: let the weekly mutation run finish, and serialise release-plz",
          "timestamp": "2026-09-06T02:05:34+09:00",
          "tree_id": "45076640d780d087a060da549715cc893b4092ac",
          "url": "https://github.com/ai-screams/scoop-uv/commit/70eb2397ca02ef6499d2ff0217d33e2b11304769"
        },
        "date": 1788628172387,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1680,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1118,
            "range": "± 3",
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
          "id": "ac604c2a13ebfde2919fbf9fcb1ccf0826f7c80f",
          "message": "Merge pull request #166 from ai-screams/ci/cache-and-action-pins\n\nci: give each workflow its real cache key, and pin the two loose actions",
          "timestamp": "2026-09-06T02:05:42+09:00",
          "tree_id": "90e016f0116fe04b590b0a19337a36f1509bbb6c",
          "url": "https://github.com/ai-screams/scoop-uv/commit/ac604c2a13ebfde2919fbf9fcb1ccf0826f7c80f"
        },
        "date": 1788628421738,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1880,
            "range": "± 26",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1326,
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
          "id": "a0ced1aff7d3585e56bc8bb4b2d66a7a08f0a43f",
          "message": "Merge pull request #163 from ai-screams/docs/ci-cd-design\n\ndocs: document the CI/CD design",
          "timestamp": "2026-09-06T02:14:46+09:00",
          "tree_id": "967b192d627c8457daaf76345805d3b2e412ccbb",
          "url": "https://github.com/ai-screams/scoop-uv/commit/a0ced1aff7d3585e56bc8bb4b2d66a7a08f0a43f"
        },
        "date": 1788628728406,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1689,
            "range": "± 67",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1123,
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
          "id": "8140def69dbe22652ff4d17140f348f86bac2067",
          "message": "Merge pull request #167 from ai-screams/fix/uv-min-version\n\nfix(uv): correct MIN_VERSION to 0.5.19 and verify the floor in CI",
          "timestamp": "2026-09-06T10:46:41+09:00",
          "tree_id": "5f1ba6ce9353b309197900d31e358aa2307c27cc",
          "url": "https://github.com/ai-screams/scoop-uv/commit/8140def69dbe22652ff4d17140f348f86bac2067"
        },
        "date": 1788659450655,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 671,
            "range": "± 12",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 491,
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
          "id": "1186a36f70269f6d6c3536c567c5bf7cc8481a39",
          "message": "Merge pull request #168 from ai-screams/fix/migrate-batch-name-collision\n\nfix(migrate): reject duplicate names within one batch; label env-var source",
          "timestamp": "2026-09-06T15:08:42+09:00",
          "tree_id": "a5b84b04061fbc772fc287e8350a321adf08ecb8",
          "url": "https://github.com/ai-screams/scoop-uv/commit/1186a36f70269f6d6c3536c567c5bf7cc8481a39"
        },
        "date": 1788675164901,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1674,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1112,
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
          "id": "8759a7c3514daab45a573ed568e88282d0a05a6e",
          "message": "Merge pull request #169 from ai-screams/release-plz-2026-09-06T01-47-21Z\n\nchore: release v0.15.3",
          "timestamp": "2026-09-06T19:01:20+09:00",
          "tree_id": "5d2d930aafd1b7945329b653d9e2aa4dc54fc939",
          "url": "https://github.com/ai-screams/scoop-uv/commit/8759a7c3514daab45a573ed568e88282d0a05a6e"
        },
        "date": 1788689152563,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1680,
            "range": "± 6",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1102,
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
          "id": "b9437c8dacc1dca6198b846917ad8d4e9c987658",
          "message": "Merge pull request #170 from ai-screams/fix/release-plz-toolchain\n\nci(release): run release-plz under stable, not the MSRV pin",
          "timestamp": "2026-09-06T19:41:21+09:00",
          "tree_id": "6f4365daa4a54007b2374775644253963ff9f9af",
          "url": "https://github.com/ai-screams/scoop-uv/commit/b9437c8dacc1dca6198b846917ad8d4e9c987658"
        },
        "date": 1788691523177,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1864,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1323,
            "range": "± 3",
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
          "id": "18a9ecee6d56bb4448d0888bf05fd3bbfa30c7ee",
          "message": "Merge pull request #171 from ai-screams/test/kill-migrate-mutants\n\ntest(migrate): close the mutation gaps the weekly run had been finding",
          "timestamp": "2026-09-10T16:31:01+09:00",
          "tree_id": "2ceb8c522dcc2ad96b6ff6537bb9ce8fe4516a44",
          "url": "https://github.com/ai-screams/scoop-uv/commit/18a9ecee6d56bb4448d0888bf05fd3bbfa30c7ee"
        },
        "date": 1789025774824,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1868,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1324,
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
          "id": "88befe7cb72a9106c48b09cba1188efd40c8ca51",
          "message": "Merge pull request #172 from ai-screams/docs/refresh-drift-2026-09\n\ndocs: refresh every doc surface against the code",
          "timestamp": "2026-09-10T23:29:28+09:00",
          "tree_id": "87ed7c7245a16ca9249abce3b33d566025a8d2cf",
          "url": "https://github.com/ai-screams/scoop-uv/commit/88befe7cb72a9106c48b09cba1188efd40c8ca51"
        },
        "date": 1789050816691,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1884,
            "range": "± 9",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1338,
            "range": "± 6",
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
          "id": "dedceef2ee04e31f45094eee47512a4e3e12fbb9",
          "message": "Merge pull request #175 from ai-screams/chore/bump-msrv-1.89\n\nfix(msrv): [#173] bump to 1.89 so the rust-dependencies group can land",
          "timestamp": "2026-09-16T00:03:54+09:00",
          "tree_id": "185298adde404931978031da9127ab073ce0f187",
          "url": "https://github.com/ai-screams/scoop-uv/commit/dedceef2ee04e31f45094eee47512a4e3e12fbb9"
        },
        "date": 1789484916530,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1666,
            "range": "± 13",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1091,
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
          "id": "51c5c1556099f64bcf896e6ad04782f166d60ddd",
          "message": "Merge pull request #177 from ai-screams/fix/release-pr-version-samples\n\nfix(release): [#176] sync version samples into the release PR automatically",
          "timestamp": "2026-09-16T13:51:59+09:00",
          "tree_id": "53dd23db65d69db4c0e30e5d7601094d65edb5a8",
          "url": "https://github.com/ai-screams/scoop-uv/commit/51c5c1556099f64bcf896e6ad04782f166d60ddd"
        },
        "date": 1789534556307,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1679,
            "range": "± 10",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1112,
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
          "id": "61402c826067419b3669b80347de6e1d9c300b37",
          "message": "Merge pull request #176 from ai-screams/release-plz-2026-09-15T15-04-23Z\n\nchore: release v0.15.4",
          "timestamp": "2026-09-16T14:00:40+09:00",
          "tree_id": "bafb3cc6aee5fcf6c4d75a9a9c5aff2a1bc39675",
          "url": "https://github.com/ai-screams/scoop-uv/commit/61402c826067419b3669b80347de6e1d9c300b37"
        },
        "date": 1789535104016,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1919,
            "range": "± 22",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1348,
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
          "id": "883f1b302492cf409afb3eb82ef14bd3bf88a039",
          "message": "Merge pull request #174 from ai-screams/dependabot/github_actions/github-actions-03db81fe05\n\nci(deps): bump astral-sh/setup-uv from 10.0.1 to 10.1.0 in the github-actions group",
          "timestamp": "2026-09-16T14:09:17+09:00",
          "tree_id": "c20e4c35308bbea4a26e74d73d7445314c5a278a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/883f1b302492cf409afb3eb82ef14bd3bf88a039"
        },
        "date": 1789535596003,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1924,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1331,
            "range": "± 50",
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
          "id": "50b29942fc9695d2436e21354be394df099a7f9e",
          "message": "Merge pull request #178 from ai-screams/fix/ko-po-and-dependabot-ignore\n\nfix(docs): regenerate ko.po for the 1.89 docs and gate serial_test 4.x",
          "timestamp": "2026-09-16T16:06:34+09:00",
          "tree_id": "2a8fde0e08d063db1b6bf4ef6b3e7e839042debf",
          "url": "https://github.com/ai-screams/scoop-uv/commit/50b29942fc9695d2436e21354be394df099a7f9e"
        },
        "date": 1789542633761,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 577,
            "range": "± 8",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 418,
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
          "id": "eb93345308cbc5d616964c29b233de3d379dfe0b",
          "message": "Merge pull request #180 from ai-screams/fix/docs-guard-on-pr\n\nci(docs): run the docs guards on PRs and drop the unmaintained serde_yaml",
          "timestamp": "2026-09-16T17:54:39+09:00",
          "tree_id": "5bf3a809c331208beea5f90f790dd99ec210bc28",
          "url": "https://github.com/ai-screams/scoop-uv/commit/eb93345308cbc5d616964c29b233de3d379dfe0b"
        },
        "date": 1789549118029,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1902,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1326,
            "range": "± 5",
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
          "id": "fe6af580c4ff9323f8bb0b74ca9e0f339cacef99",
          "message": "Merge pull request #181 from ai-screams/fix/api-md-date-stamp\n\ndocs(api): drop the ambiguous Last Updated stamp from the API reference",
          "timestamp": "2026-09-16T22:43:01+09:00",
          "tree_id": "583f67b9a4a6e1993621f3da02ced8c1350aeb03",
          "url": "https://github.com/ai-screams/scoop-uv/commit/fe6af580c4ff9323f8bb0b74ca9e0f339cacef99"
        },
        "date": 1789566427463,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1925,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1336,
            "range": "± 6",
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
          "id": "04d3d72189f4264e1d27fe49bc1e6aa1c04ca935",
          "message": "Merge pull request #182 from ai-screams/fix/saphyr-and-docs-check\n\nrefactor(ci): split PR docs checks from deploy and use the already-locked serde-saphyr",
          "timestamp": "2026-09-16T22:43:41+09:00",
          "tree_id": "949df8abb3034e4ff97c03545424db169589e3c7",
          "url": "https://github.com/ai-screams/scoop-uv/commit/04d3d72189f4264e1d27fe49bc1e6aa1c04ca935"
        },
        "date": 1789566676696,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1695,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1116,
            "range": "± 62",
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
          "id": "b3bafed1c5a72531b3dc5a0fcb93e06ce5693031",
          "message": "Merge pull request #184 from ai-screams/ci/codecov-thresholds\n\nci(coverage): state a coverage target instead of reporting into the void",
          "timestamp": "2026-09-17T00:00:20+09:00",
          "tree_id": "ef09f19218223d17a2727df3befa650de77c9da4",
          "url": "https://github.com/ai-screams/scoop-uv/commit/b3bafed1c5a72531b3dc5a0fcb93e06ce5693031"
        },
        "date": 1789571067335,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1906,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1317,
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
          "id": "9a9241dae093ae0e351d27a0c7f21151c86b92c3",
          "message": "Merge pull request #185 from ai-screams/ci/stable-msrv-job-name\n\nci(msrv): drop the version from the MSRV job name",
          "timestamp": "2026-09-17T00:27:23+09:00",
          "tree_id": "a310dbc41370bfb8f3cfda96689f820ea9ea4f87",
          "url": "https://github.com/ai-screams/scoop-uv/commit/9a9241dae093ae0e351d27a0c7f21151c86b92c3"
        },
        "date": 1789572658505,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1343,
            "range": "± 39",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 893,
            "range": "± 39",
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
          "id": "cd592c4fd7dcd73f9259f68ef083b712616d8c91",
          "message": "Merge pull request #186 from ai-screams/fix/release-sync-ko-po\n\nfix(release): keep ko.po in sync when the release PR rewrites version samples",
          "timestamp": "2026-09-17T01:24:48+09:00",
          "tree_id": "49a7b7200c2a38e7118dac38c1fc8d20a3249335",
          "url": "https://github.com/ai-screams/scoop-uv/commit/cd592c4fd7dcd73f9259f68ef083b712616d8c91"
        },
        "date": 1789576109598,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1481,
            "range": "± 60",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1034,
            "range": "± 3",
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
          "id": "37ab23726ae47026e9ab66c778dc85d64d0c108f",
          "message": "Merge pull request #187 from ai-screams/ci/bench-threshold-from-data\n\nci(bench): set the CPU benchmark threshold from measured variance",
          "timestamp": "2026-09-17T01:36:21+09:00",
          "tree_id": "13f36edb919da7444a771fc9b553cef6268db6cd",
          "url": "https://github.com/ai-screams/scoop-uv/commit/37ab23726ae47026e9ab66c778dc85d64d0c108f"
        },
        "date": 1789576823407,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1690,
            "range": "± 11",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1115,
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
          "id": "8e427f71677c38c7bdc65a6343a2a4cfc2ae9bdf",
          "message": "Merge pull request #183 from ai-screams/release-plz-2026-09-16T13-44-01Z\n\nchore: release v0.15.5",
          "timestamp": "2026-09-17T01:45:18+09:00",
          "tree_id": "675ea5938c914da8f1f110e168ddf4334d9146b3",
          "url": "https://github.com/ai-screams/scoop-uv/commit/8e427f71677c38c7bdc65a6343a2a4cfc2ae9bdf"
        },
        "date": 1789577361119,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1682,
            "range": "± 17",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1115,
            "range": "± 8",
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
          "id": "7a305a8a523266e0a8320d0c04f69cba19e26f64",
          "message": "Merge pull request #189 from ai-screams/ci/coverage-floor\n\nci(coverage): enforce a floor locally, since codecov/project never posts",
          "timestamp": "2026-09-17T11:29:08+09:00",
          "tree_id": "eac2070c27225a47d9d90f81005aa4fbca2df73a",
          "url": "https://github.com/ai-screams/scoop-uv/commit/7a305a8a523266e0a8320d0c04f69cba19e26f64"
        },
        "date": 1789612382132,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1698,
            "range": "± 15",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1113,
            "range": "± 5",
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
          "id": "808369b218a367f75f3e79a72638f1bae294d14b",
          "message": "docs(theme): add canonical/hreflang links and an Ai-Scream home link (#193)\n\n* docs(theme): add canonical/hreflang links and an Ai-Scream home link\n\nThe docs live at https://ai-scream.ai/scoop-uv/ next to the Ai-Scream\nhome page, which now lists them in its sitemap. This makes the docs\nside say so too:\n\n- per-page <link rel=\"canonical\"> on https://ai-scream.ai, with\n  /x/index.html folded into /x/, plus hreflang en/ko/x-default pairing\n  each English page with its Korean twin. Set from the head script\n  because mdBook only exposes the source .md path to the theme\n- a \"by Ai-Scream\" link (to /ko/ on Korean pages) in the locale\n  switcher\n- the switcher now sits in the menu bar's right-button row instead of\n  floating over it: the fixed overlay covered the print, git and edit\n  buttons (already true before this change; the longer label made it\n  worse). Links take the theme's icon colors; below 620px only the\n  locale links remain\n- the locale-path logic is shared by the switcher and the SEO links\n\nCloses #192\n\n* docs(theme): keep 404 and print pages out of canonical/hreflang clusters\n\nGitHub Pages serves 404.html for any missing URL, so deriving the links\nfrom location.pathname declared /scoop-uv/missing canonical and\nadvertised /scoop-uv/ko/missing as its Korean twin; a direct\n/404.html request (HTTP 200) got the same. print.html, which mdBook\nalready marks noindex, also got the links.\n\n- the template context's path (404.md / print.md) sets skipSeoLinks,\n  so those pages carry no canonical or hreflang\n- 404.html gets a static <meta name=\"robots\" content=\"noindex\">\n\nChecked in Chrome against a Pages-like server: missing URLs, /404.html\nand /print.html have 0 canonical/hreflang links and noindex; content\npages still get 1 canonical + 3 hreflang. Forcing skipSeoLinks=false\nbrings the links back on the 404 responses.",
          "timestamp": "2026-09-26T00:59:17+09:00",
          "tree_id": "8bbf6513365a66044d096690738f9b416a222dad",
          "url": "https://github.com/ai-screams/scoop-uv/commit/808369b218a367f75f3e79a72638f1bae294d14b"
        },
        "date": 1790352196618,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1695,
            "range": "± 5",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1120,
            "range": "± 4",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jsmomo0305@gmail.com",
            "name": "sayam-1",
            "username": "sayam-1"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "a62dd6e316da233fe7e8d1f9c1326cba3b898378",
          "message": "feat(i18n): add Spanish translation (#190)\n\nAdds the `es` locale: every key in locales/app.yml, SUPPORTED_LANGS,\nthe i18n completeness gate, and the `scuv lang` completion lists in all\nfour shells.\n\nCo-authored-by: pignuante <hanyul.ryu@hanyul.xyz>",
          "timestamp": "2026-09-26T03:56:26+09:00",
          "tree_id": "f4a3726ff6059c62081ec4a08478d68c922c514e",
          "url": "https://github.com/ai-screams/scoop-uv/commit/a62dd6e316da233fe7e8d1f9c1326cba3b898378"
        },
        "date": 1790362832468,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 2040,
            "range": "± 20",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1409,
            "range": "± 3",
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
          "id": "c1fc3e917acf73edfc709d6dca1a68de6023b3b5",
          "message": "refactor(compat)!: drop the scoop-era legacy fallbacks\n\nThe 0.15.0 rename kept the scoop-era names readable behind one-shot\nwarnings that promised removal in 0.16.0. This is that removal: the\nSCOOP_* environment variables, ~/.scoop, .scoop-version and .scoop.toml\nare no longer read, the bash/zsh/fish `scoop` forwarder is gone, and\n`scuv shell` exports SCUV_VERSION only. The doctor `legacy` check stays\nas a warn-only diagnostic so an incomplete upgrade is not silent.\n.scoop-metadata.json and scoop_export_version are unchanged (on-disk\nformats).\n\nBREAKING CHANGE: SCOOP_HOME, SCOOP_VERSION, SCOOP_LANG,\nSCOOP_RESOLVE_MAX_DEPTH and SCOOP_NO_AUTO are ignored, as are ~/.scoop,\n.scoop-version and .scoop.toml. Rename them to the SCUV_* / .scuv names\nand run `mv ~/.scoop ~/.scuv`. SCUV_SUPPRESS_DEPRECATION is no longer\nrecognised.",
          "timestamp": "2026-09-26T09:47:37+09:00",
          "tree_id": "e44f9e574d14fe3a3575fdef690b9c772909030e",
          "url": "https://github.com/ai-screams/scoop-uv/commit/c1fc3e917acf73edfc709d6dca1a68de6023b3b5"
        },
        "date": 1790383902331,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1885,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1327,
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
          "id": "96aca282f82541cb55e56882eed7e41332d38e1f",
          "message": "Merge pull request #194 from ai-screams/release-plz-2026-09-25T18-56-49Z\n\nchore: release v0.16.0",
          "timestamp": "2026-09-26T09:59:49+09:00",
          "tree_id": "0b4e84f13dc1338c9dffb5017ff9710ef08a39de",
          "url": "https://github.com/ai-screams/scoop-uv/commit/96aca282f82541cb55e56882eed7e41332d38e1f"
        },
        "date": 1790384653992,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1686,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1130,
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
          "id": "12817f5f64c22b18118c63172aef98e2189a8032",
          "message": "docs(i18n): fix the add-a-locale checklist and guard the language lists\n\nFixes the add-a-locale guidance that skipped the bash and PowerShell completion lists (#190), lists Spanish everywhere the supported languages are enumerated, pins each shell's completion list to SUPPORTED_LANGS with a test, guards the doc lists in check-doc-references.py, transcribes the lang command examples from the binary, brings ci-cd.md up to date with docs-check.yml, and records the merge and release conventions in CLAUDE.md.",
          "timestamp": "2026-09-26T11:13:43+09:00",
          "tree_id": "0a3662872ad0836d6b47a1676397124516bbcce5",
          "url": "https://github.com/ai-screams/scoop-uv/commit/12817f5f64c22b18118c63172aef98e2189a8032"
        },
        "date": 1790389064486,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1690,
            "range": "± 25",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1122,
            "range": "± 11",
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
          "id": "df7ad0a5c6798541102f1436d5cefa33c2a77628",
          "message": "ci(cache): write caches only from main and tolerate export failures\n\nThe Actions cache reached 10.49 GB against the 10 GB allowance and LRU eviction failed a green Docker build during a BuildKit export (error writing layer blob: not_found). rust-cache steps now save only from main, BuildKit cache-to is emitted only off pull_request with ignore-error=true, and cache-cleanup.yml deletes a PR's caches when it closes.",
          "timestamp": "2026-09-26T16:51:33+09:00",
          "tree_id": "2eb9867cd0abf499e0a81c5f7169cf81a9914844",
          "url": "https://github.com/ai-screams/scoop-uv/commit/df7ad0a5c6798541102f1436d5cefa33c2a77628"
        },
        "date": 1790409324754,
        "tool": "cargo",
        "benches": [
          {
            "name": "find_executable_in_hit",
            "value": 1691,
            "range": "± 7",
            "unit": "ns/iter"
          },
          {
            "name": "find_executable_in_miss",
            "value": 1114,
            "range": "± 12",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}