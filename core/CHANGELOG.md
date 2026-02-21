# Changelog

## [0.3.0](https://github.com/Its-Satyajit/dev-bind/compare/devbind-core-v0.2.2...devbind-core-v0.3.0) (2026-02-21)


### Features

* **ui:** center main content, add footer, bump core & deps, and make CLI version optional ([7567089](https://github.com/Its-Satyajit/dev-bind/commit/7567089eed420af82e43f69684b16774547e185a))
* **ui:** center main content, add footer, bump core & deps, and make CLI version optional ([101d2ba](https://github.com/Its-Satyajit/dev-bind/commit/101d2ba31ca4a23c9def15d2d3f583ec0fbf4807))

## [0.2.2](https://github.com/Its-Satyajit/dev-bind/compare/devbind-core-v0.2.1...devbind-core-v0.2.2) (2026-02-21)


### Bug Fixes

* **deps:** update rust crate thiserror to v2 ([#16](https://github.com/Its-Satyajit/dev-bind/issues/16)) ([d1b82f1](https://github.com/Its-Satyajit/dev-bind/commit/d1b82f1b490b23e7ab28682b753b25406ce4f034))
* update core to support rcgen 0.14 API ([d9c990e](https://github.com/Its-Satyajit/dev-bind/commit/d9c990e78b53ea82d6a66105c8deef648c5c936c))

## [0.2.1](https://github.com/Its-Satyajit/dev-bind/compare/devbind-core-v0.2.0...devbind-core-v0.2.1) (2026-02-21)


### Bug Fixes

* **deps:** update rust crate toml to v1 ([#17](https://github.com/Its-Satyajit/dev-bind/issues/17)) ([be9d256](https://github.com/Its-Satyajit/dev-bind/commit/be9d25651796a06e11591e9de7287d467a44e37a))

## [0.2.0](https://github.com/Its-Satyajit/dev-bind/compare/devbind-core-v0.1.0...devbind-core-v0.2.0) (2026-02-21)


### Features

* add ephemeral run support and route management APIs ([0577a1c](https://github.com/Its-Satyajit/dev-bind/commit/0577a1ccad3416dfaddd0e01844a243e5d1696d2))
* add Root CA trust install command and UI button ([486572d](https://github.com/Its-Satyajit/dev-bind/commit/486572d130a4efab6029f80be7b7d42c5c673032))
* add root CA uninstallation and update README ([808dddc](https://github.com/Its-Satyajit/dev-bind/commit/808dddc2aeec076331528bf92255e70088fa5cea))
* **cli:** expose subcommand modules and remove hosts tests ([4fb0845](https://github.com/Its-Satyajit/dev-bind/commit/4fb0845b68c6465793e3e4b9b772807805e89f28))
* **config:** add ephemeral flag and ephemeral route handling to routes ([02d9165](https://github.com/Its-Satyajit/dev-bind/commit/02d9165631491f3b94c71bc8a9ff063f7955246b))
* **config:** simplify config model, add serde defaults, and ([0c117a9](https://github.com/Its-Satyajit/dev-bind/commit/0c117a9d2b6063cec64eccac4e54ec4c456124ff))
* **dns:** change DNS bind to NetworkManager devbind0 at 127.0.2.1:53 to avoid using systemd-resolved drop-in and fix AdGuard conflicts ([a897d1b](https://github.com/Its-Satyajit/dev-bind/commit/a897d1b93a886fa66b5a1b159a1e68d577bfba71))
* document `devbind run` usage and handle HTTP upgrades in proxy ([3ec442d](https://github.com/Its-Satyajit/dev-bind/commit/3ec442d05eb5b0843178e26b422d1e0b51eb93ab))
* **gui/core:** add live UI polling, PartialEq derives, and simplify DNS check ([6ae4e54](https://github.com/Its-Satyajit/dev-bind/commit/6ae4e542a8ff0721aeed548a3f8d5a372d563edb))
* **logging:** add structured tracing to CLI, GUI, and core ([26d7152](https://github.com/Its-Satyajit/dev-bind/commit/26d7152f5cefc966616b2e3cf21b81f728b34cba))
* **logging:** add structured tracing to CLI, GUI, and core ([f91408e](https://github.com/Its-Satyajit/dev-bind/commit/f91408e99d97357c16bdea4f3a989ef3734f77a2))
* **proxy:** add HTTP-&gt;HTTPS redirect and hot-reload routes ([46e0ee9](https://github.com/Its-Satyajit/dev-bind/commit/46e0ee958a0aa42a668d0312c4e28d0dee545913))
* **proxy:** add TLS SNI proxying to local backends ([6fe7788](https://github.com/Its-Satyajit/dev-bind/commit/6fe778876767dffe15001c7841b6daeb321567dc))
* **proxy:** normalize host lookup, improve unknown-host logging, add UI tweaks and deps updates ([35cbb4c](https://github.com/Its-Satyajit/dev-bind/commit/35cbb4c7f3207955390f084a80c2582cfcaab4df))
* **trust:** pass cert path as arg and harden install/uninstall scripts ([78fa051](https://github.com/Its-Satyajit/dev-bind/commit/78fa05146760f53370ac5ddce91850eb906ff4dd))
* **trust:** pass cert path as arg and harden install/uninstall scripts ([89d7ed9](https://github.com/Its-Satyajit/dev-bind/commit/89d7ed9d919a7f2863e477eb24e45ab5e4579b35))
* **ui:** add theme support and UI config with styling ([9f402ba](https://github.com/Its-Satyajit/dev-bind/commit/9f402bac02fdba6427571e1a993e381ac17c97ba))


### Bug Fixes

* **trust:** embed secure install script using env var for cert path ([4df6b33](https://github.com/Its-Satyajit/dev-bind/commit/4df6b33c86138a324338381535e317251c570b4c))
* **trust:** embed secure install script using env var for cert path ([eddb01b](https://github.com/Its-Satyajit/dev-bind/commit/eddb01b29f36c58fbfe9471760cb5d303eb9f582))
