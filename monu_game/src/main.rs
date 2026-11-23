use monu_engine::{EmptyHooks, EngineConfig, EngineRunner};

fn main() {
    let config = EngineConfig {
        title: "MonuGame".to_string(),
        width: 1280,
        height: 720,
    };

    // Передаем EmptyHooks — никакого редактора в сборке
    EngineRunner::run(config, EmptyHooks).expect("Ошибка запуска игры");
}
