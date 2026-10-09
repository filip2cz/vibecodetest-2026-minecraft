# Vibecraft (Minecraft klon v Rustu)

Jednoduchý klon legendární hry Minecraft vytvořený v jazyce Rust pomocí moderního herního enginu **Bevy** (verze 0.19.1).

*(For English instructions, see [README.md](README.md))*

## Funkce
- Optimalizované vykreslování voxelového terénu (Greedy Meshing)
- Fyzika a kolize (hráč má správné Minecraftové rozměry a umí skákat)
- Nekonečné těžení a stavění držením tlačítka myši
- Vizuální zaměřování (highlighting)
- Různé druhy bloků (tráva, hlína, kámen, dřevo) a poloprůhledné sklo
- Uživatelské rozhraní (Hotbar) a výběr bloků scrollováním

---

## 🚀 Jak hru spustit (Obecný návod)

Pokud používáte běžnou Linuxovou distribuci (Ubuntu, Arch, Fedora, atd.), macOS nebo Windows, postupujte takto:

1. **Nainstalujte si Rust** (nejlépe přes [rustup.rs](https://rustup.rs/)).
2. Zkontrolujte, zda máte v systému nainstalované herní závislosti potřebné pro engine Bevy (pokud používáte Linux):
   - Alsa (zvuk)
   - Udev
   - Wayland / X11 a Vulkan
   - C/C++ kompilátor a pkg-config
3. Otevřete terminál ve složce s tímto projektem.
4. Hru zkompilujte a spusťte příkazem:
   ```bash
   cargo run --release
   ```
*(Poznámka: První kompilace může trvat déle kvůli stahování a překladu enginu Bevy. Přepínač `--release` zajistí maximální výkon.)*

---

## ❄️ Jak hru spustit v NixOS

NixOS nezpracovává dynamické knihovny standardní cestou, proto standardní `cargo run` na čistém systému většinou nenajde knihovny pro zobrazení okna (např. `libxkbcommon.so`). 

Projekt už v sobě ale obsahuje připravený soubor `shell.nix`, který toto automaticky řeší za vás!

1. Otevřete terminál ve složce s tímto projektem.
2. Spusťte herní prostředí pomocí:
   ```bash
   nix-shell --run "cargo run --release"
   ```
Tento příkaz dočasně nainstaluje všechny závislosti, nastaví cesty ke grafickým knihovnám (`LD_LIBRARY_PATH`) a hru nastartuje. Není potřeba nic konfigurovat ručně!

---

## 🎮 Ovládání

- **W, A, S, D:** Pohyb postavy
- **Mezerník (Space):** Skok
- **Myš (pohyb):** Rozhlížení
- **Levé tlačítko myši:** Kliknutí uzamkne kurzor do okna (potřeba k rozhlížení). Následně těží/ničí bloky (lze držet).
- **Pravé tlačítko myši:** Pokládá vybraný blok do světa (lze držet).
- **Kolečko myši:** Scrollováním vybíráte typ bloku v inventáři (Hotbaru).
- **Číslice 1 až 5:** Rychlý výběr konkrétních bloků.
- **Esc (Escape):** Odemkne a zviditelní kurzor myši (např. pro zavření hry).
