# Monu — принятые решения и границы проекта

Этот документ фиксирует результаты обсуждения. Если решение меняется, его следует обновить здесь с причиной и датой.

## Архитектура

| Область | Решение |
|---|---|
| Основной язык | Rust |
| ECS | Bevy ECS |
| Blueprint | Визуальный слой над реальными системами, не эмуляция Rust |
| Пользовательские типы | Struct, Enum, вложенные Struct, динамическая регистрация |
| Blueprint inheritance | Да, включая привязку Rust-класса/реализации |
| Замена систем | Готовые системы можно расширять или заменять через Rust |
| Плагины | Да, расширяют Editor и Engine |
| Обратная совместимость | Стараться сохранять; при неизбежных изменениях — миграции |
| Engine Version Manager | Не нужен как отдельная система |

## Rendering

| Область | Решение |
|---|---|
| Graphics API | Vulkan |
| Renderer | Forward Rendering |
| PBR | Да |
| GI | Выбор Baked / Real-time / Hybrid / No GI |
| Материалы | Node-based Material Editor |
| Пользовательские шейдеры | WGSL |
| Shader Manager | Компиляция, cache, hot reload, diagnostics |
| Post Processing | Да |
| VFX | Да, с пользовательскими шейдерами |
| Deferred Rendering | Не входит в текущий базовый план |

## Мир и окружение

| Область | Решение |
|---|---|
| Big Space | Уже есть в проекте; не дублировать |
| World Streaming | Добавить поверх Big Space |
| Terrain | Ручное редактирование и импорт внешних Heightmap |
| Terrain masks | Заложить импорт/редактирование дополнительных карт |
| Foliage | Да |
| Water | Реки, озёра, моря, водопады и подводная среда |
| Caves | Полноценная 3D-геометрия, многоуровневость |
| Mesh modeling | Внешние инструменты; не делать полноценный Mesh Editor |

## Генераторы

- Несколько специализированных генераторов вместо одного универсального.
- Architecture, Interior, Road & Infrastructure, Nature & Environment, Cave, City.
- Terrain Generator — дополнительный инструмент.
- Generator Graph — да, с эффективным исполнением и пользовательскими Rust-узлами.
- Road Generator: настройка контрольных точек/кривой и возможность зафиксировать результат.
- Отдельный Dungeon Generator — нет; использовать комбинацию Cave, Interior и Architecture.
- Planet & Space Generator — не включать сейчас.

## Gameplay и симуляция

- Physics library пока не выбрана; сначала сравнение и прототип.
- Destruction: pre-fractured, runtime fracturing и hybrid.
- Save System сохраняет состояние разрушений.
- AI: Behavior Trees, State Machine, Decision System.
- AI LOD для удалённых NPC.
- Navigation: ground, air, water и custom.
- Crowd System — да.
- Inventory — да, но как готовая расширяемая подсистема.
- Dialogue — да.
- Quest System — опционально.
- Building System — опционально.
- Interaction — набор готовых пресетов.
- Movement: riding, flight, swimming/diving; транспорт можно расширять.
- Physics-Based Animation — предусмотреть.
- Audio, VFX, UI, Localization и Cinematic — да.

## Networking

- Networking Framework предусмотреть.
- Multiplayer выключен по умолчанию.
- ECS Replication, RPC, authoritative server, physics sync, network events.
- Компоненты реплицируются по явному выбору разработчика.
- Anti-Cheat — отложить.

## Editor и workflow

- Модульный редактор с dockable panels и workspace layouts.
- Несколько сцен и окон.
- Переназначаемые горячие клавиши.
- Project Manager — да.
- Asset Manager — поверх обычной файловой системы.
- Presets System — да.
- Plugin System — да.
- Plugin Marketplace — нет; ручная установка.
- Lore integration — опционально, без жёсткой зависимости.
- Git integration — не нужна.
- Documentation System — предусмотреть.
- Testing Framework — отдельный будущий инструмент.
- Build & Packaging — стандартные Rust/Cargo инструменты.

## Asset Pipeline

- Разработчик выбирает Automatic, Manual или Hybrid.
- Профили импорта и ручное переопределение настроек.
- Кэширование и повторный импорт.
- Несколько вариантов ресурсов для платформ и уровней качества.
- LOD и варианты сжатия.
- Исходные файлы остаются обычными файлами проекта.

## Не включать в текущую цель

- VR/AR — не планировать специально сейчас; общая расширяемость через плагины достаточна.
- Modding Framework — отложить.
- Anti-Cheat — отложить.
- World Economy — не включать.
- Полную Life Simulation NPC — не делать обязательной; возможно как опциональный модуль/плагин.
- Planet Generator — не включать.
- Отдельный Dungeon Generator — не включать.
- Marketplace — не создавать.
- Полноценный Mesh Editor — не создавать.
