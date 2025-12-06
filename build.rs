use winres::WindowsResource;

fn main() {
    // Führe das nur auf Windows aus
    if cfg!(target_os = "windows") {
        let mut res = WindowsResource::new();
        // Hier sagen wir ihm, welches Icon er nehmen soll
        // Die Datei "icon.ico" muss im Hauptverzeichnis liegen
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}