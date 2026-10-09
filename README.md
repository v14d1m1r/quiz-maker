# Kviz — konzolna Rust aplikacija

Pokretanje iz direktorijuma projekta:

```sh
cargo run
```

Potrebni su Rust 1.88 ili noviji i Cargo.

## Upotreba

1. Izaberite **Dodaj kategoriju** i upišite naziv, npr. `Nauka`.
2. U otvorenoj kategoriji izaberite **Dodaj potkategoriju**, npr. `Matematika`.
3. U potkategoriji izaberite **Dodaj pitanja**.
4. Upišite pitanje i izaberite tip: jedan ili više tačnih odgovora.
5. Unesite bar dva različita ponuđena odgovora, svaki u zasebnom redu. Pritisnite Enter na praznom redu da završite listu.
6. Označite tačne odgovore brojevima: `2` za jedan, odnosno `1,3` za više tačnih odgovora. Tip „više tačnih odgovora” zahteva najmanje dva tačna odgovora.
7. Nastavite sa sledećim pitanjem ili pritisnite Enter na praznom tekstu pitanja za povratak.

Postojećim podacima pristupate kroz **Otvori kategoriju**, a zatim **Otvori potkategoriju**. U potkategoriji su dostupni pregled sa označenim tačnim odgovorima i rešavanje kviza. Pitanje se boduje kao tačno samo ako su izabrani svi tačni odgovori bez netačnih; redosled izbora nije važan. Kviz prikazuje pitanja redom unosa i završni rezultat.

Svaka kategorija, potkategorija i završeno pitanje automatski se čuvaju u `quiz-data.json` u trenutnom radnom direktorijumu. Pokrećite aplikaciju iz istog direktorijuma da biste koristili iste podatke. Fajl možete kopirati radi rezervne kopije. Ne pokrećite više instanci istovremeno nad istim fajlom. Neispravan fajl izaziva grešku pri pokretanju i ne prepisuje se. Nezavršeno pitanje se ne čuva.

Nazivi kategorija moraju biti jedinstveni, a nazivi potkategorija jedinstveni unutar svoje kategorije, bez obzira na velika/mala slova. Izmena i brisanje još nisu deo ove početne verzije.

## Provera

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```
