
# Планы развития
* модуль генерации растительностий
* гафовый редактор логики аналог bluprint
* редактор материалов
* пред просмотров ассетов
* редактор анимации
* редактор ланшафта
* генератор/редактор карты 2д
* настройки уровня.

```
monu_engine/src/
├── lib.rs
├── app.rs                  # Только сборка App и спавн базовой сцены
├── settings/               # Модуль настроек уровня (данные + логика + плагин)
│   ├── mod.rs              # Ресурс LevelSettings, JSON-сериализация и LevelSettingsPlugin
│   ├── space.rs            # WorldSpaceSettings (чанки, BigSpace, стриминг)
│   └── environment.rs      # EnvironmentSettings (небо, солнце, туман, физика)
├── gizmos.rs               # Сетка редактора, оси и рамка границ мира
└── commands.rs             # EditorCommand и обработчик команд сцены

monu_editor/src/
├── main.rs
└── ui/                     # Модуль интерфейса редактора
    ├── mod.rs              # EditorUiPlugin, DockState и главное меню
    ├── outliner.rs         # Вкладка списка объектов (Outliner)
    ├── details.rs          # Вкладка инспектора (Details)
    └── world_settings.rs   # Вкладка настроек уровня (World Settings)
```
