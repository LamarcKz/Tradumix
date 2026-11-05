use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use nucleo::GrupoDeProcessos;

/// Se o app fecha (o grupo é liberado), o processo filho tem que acabar junto.
#[test]
fn liberar_o_grupo_encerra_o_processo() {
    let mut filho = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let grupo = GrupoDeProcessos::novo().unwrap();
    grupo.incluir(&filho).unwrap();
    drop(grupo);

    let inicio = Instant::now();
    while inicio.elapsed() < Duration::from_secs(5) {
        if filho.try_wait().unwrap().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = filho.kill();
    panic!("o processo continuou rodando depois de liberar o grupo");
}
