window.BENCHMARK_DATA = {
  "lastUpdate": 1781590405459,
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
      }
    ]
  }
}