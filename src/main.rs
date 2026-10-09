use quiz_maker::{AnswerMode, Category, Question, Quiz, Subcategory, parse_selection, same_name};
use std::{
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

const DATA_FILE: &str = "quiz-data.json";

fn input(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut line = String::new();
    if io::stdin().read_line(&mut line)? == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Kraj unosa"));
    }
    Ok(line.trim().to_string())
}

fn required(prompt: &str) -> io::Result<String> {
    loop {
        let value = input(prompt)?;
        if !value.is_empty() {
            return Ok(value);
        }
        println!("Unos ne sme biti prazan.");
    }
}

fn number(prompt: &str, min: usize, max: usize) -> io::Result<usize> {
    loop {
        if let Ok(n) = input(prompt)?.parse::<usize>()
            && (min..=max).contains(&n)
        {
            return Ok(n);
        }
        println!("Unesite broj od {min} do {max}.");
    }
}

fn selection(count: usize, mode: AnswerMode) -> io::Result<Vec<usize>> {
    loop {
        match parse_selection(
            &input("Broj odgovora (više brojeva odvojite zarezom): ")?,
            count,
            mode,
        ) {
            Ok(indices) => return Ok(indices),
            Err(e) => println!("{e}"),
        }
    }
}

fn save(quiz: &Quiz) -> io::Result<()> {
    quiz.save(Path::new(DATA_FILE))
        .map_err(|e| io::Error::other(format!("Čuvanje nije uspelo: {e}")))
}

fn add_question() -> io::Result<Option<Question>> {
    let text = input("\nTekst pitanja (Enter za povratak): ")?;
    if text.is_empty() {
        return Ok(None);
    }
    println!("1. Jedan tačan odgovor\n2. Više tačnih odgovora (najmanje dva)");
    let mode = match number("Tip pitanja: ", 1, 2)? {
        1 => AnswerMode::Single,
        _ => AnswerMode::Multiple,
    };
    println!("Unesite ponuđene odgovore redom. Prazan unos završava listu.");
    let mut answers = Vec::<String>::new();
    loop {
        let answer = input(&format!("Odgovor {}: ", answers.len() + 1))?;
        if answer.is_empty() {
            if answers.len() >= 2 {
                break;
            }
            println!("Unesite bar dva odgovora.");
        } else if answers.iter().any(|a| same_name(a, &answer)) {
            println!("Taj odgovor već postoji.");
        } else {
            answers.push(answer);
        }
    }
    println!("Označite tačne odgovore.");
    let correct = selection(answers.len(), mode)?;
    Ok(Some(Question {
        text,
        mode,
        answers,
        correct,
    }))
}

fn show_question(question: &Question, reveal: bool) {
    let label = match question.mode {
        AnswerMode::Single => "jedan tačan odgovor",
        AnswerMode::Multiple => "više tačnih odgovora",
    };
    println!("{} ({label})", question.text);
    for (i, answer) in question.answers.iter().enumerate() {
        let marker = if reveal && question.correct.contains(&i) {
            " [tačno]"
        } else {
            ""
        };
        println!("  {}. {answer}{marker}", i + 1);
    }
}

fn play(subcategory: &Subcategory) -> io::Result<()> {
    if subcategory.questions.is_empty() {
        println!("Još nema pitanja u ovoj potkategoriji.");
        return Ok(());
    }
    let mut score = 0;
    for (i, question) in subcategory.questions.iter().enumerate() {
        println!("\nPitanje {}/{}", i + 1, subcategory.questions.len());
        show_question(question, false);
        if question.is_correct(&selection(question.answers.len(), question.mode)?) {
            println!("Tačno!");
            score += 1;
        } else {
            println!("Netačno. Tačni odgovori:");
            for &index in &question.correct {
                println!("  {}. {}", index + 1, question.answers[index]);
            }
        }
    }
    println!(
        "\nRezultat: {score}/{} tačno rešenih pitanja.",
        subcategory.questions.len()
    );
    Ok(())
}

fn subcategory_menu(quiz: &mut Quiz, c: usize, s: usize) -> io::Result<()> {
    loop {
        let category = &quiz.categories[c];
        let sub = &category.subcategories[s];
        println!(
            "\n{} / {} ({} pitanja)",
            category.name,
            sub.name,
            sub.questions.len()
        );
        println!(
            "1. Dodaj pitanja\n2. Pregled pitanja i tačnih odgovora\n3. Pokreni kviz\n0. Nazad"
        );
        match number("Izbor: ", 0, 3)? {
            0 => return Ok(()),
            1 => {
                while let Some(question) = add_question()? {
                    quiz.categories[c].subcategories[s].questions.push(question);
                    save(quiz)?;
                    println!("Pitanje je sačuvano. Možete uneti sledeće pitanje.");
                }
            }
            2 => {
                if sub.questions.is_empty() {
                    println!("Još nema pitanja u ovoj potkategoriji.");
                }
                for (i, question) in sub.questions.iter().enumerate() {
                    println!("\nPitanje {}:", i + 1);
                    show_question(question, true);
                }
            }
            3 => play(sub)?,
            _ => unreachable!(),
        }
    }
}

fn category_menu(quiz: &mut Quiz, c: usize) -> io::Result<()> {
    loop {
        let category = &quiz.categories[c];
        println!("\nKategorija: {}", category.name);
        println!("1. Dodaj potkategoriju\n2. Otvori potkategoriju\n0. Nazad");
        match number("Izbor: ", 0, 2)? {
            0 => return Ok(()),
            1 => {
                let name = required("Naziv potkategorije: ")?;
                if category
                    .subcategories
                    .iter()
                    .any(|s| same_name(&s.name, &name))
                {
                    println!("Potkategorija sa tim nazivom već postoji u ovoj kategoriji.");
                    continue;
                }
                let s = category.subcategories.len();
                quiz.categories[c].subcategories.push(Subcategory {
                    name,
                    questions: vec![],
                });
                save(quiz)?;
                println!("Potkategorija je sačuvana.");
                subcategory_menu(quiz, c, s)?;
            }
            2 => {
                if category.subcategories.is_empty() {
                    println!("Prvo dodajte potkategoriju.");
                    continue;
                }
                for (i, sub) in category.subcategories.iter().enumerate() {
                    println!("{}. {} ({} pitanja)", i + 1, sub.name, sub.questions.len());
                }
                let n = number(
                    "Potkategorija (0 za nazad): ",
                    0,
                    category.subcategories.len(),
                )?;
                if n > 0 {
                    subcategory_menu(quiz, c, n - 1)?;
                }
            }
            _ => unreachable!(),
        }
    }
}

fn run() -> io::Result<()> {
    let mut quiz = Quiz::load(Path::new(DATA_FILE)).map_err(|e| {
        io::Error::other(format!(
            "Učitavanje fajla {DATA_FILE} nije uspelo: {e}. Postojeći fajl nije izmenjen."
        ))
    })?;
    println!("Kviz — kreiranje i rešavanje pitanja\nPodaci: {DATA_FILE} (automatsko čuvanje)");
    loop {
        println!("\n1. Dodaj kategoriju\n2. Otvori kategoriju\n0. Izlaz");
        match number("Izbor: ", 0, 2)? {
            0 => return Ok(()),
            1 => {
                let name = required("Naziv kategorije: ")?;
                if quiz.categories.iter().any(|c| same_name(&c.name, &name)) {
                    println!("Kategorija sa tim nazivom već postoji.");
                    continue;
                }
                let c = quiz.categories.len();
                quiz.categories.push(Category {
                    name,
                    subcategories: vec![],
                });
                save(&quiz)?;
                println!("Kategorija je sačuvana.");
                category_menu(&mut quiz, c)?;
            }
            2 => {
                if quiz.categories.is_empty() {
                    println!("Prvo dodajte kategoriju.");
                    continue;
                }
                for (i, category) in quiz.categories.iter().enumerate() {
                    println!("{}. {}", i + 1, category.name);
                }
                let n = number("Kategorija (0 za nazad): ", 0, quiz.categories.len())?;
                if n > 0 {
                    category_menu(&mut quiz, n - 1)?;
                }
            }
            _ => unreachable!(),
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => {
            println!("Doviđenja!");
            ExitCode::SUCCESS
        }
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
            println!("\nKraj unosa. Svi završeni unosi su sačuvani.");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Greška: {e}");
            ExitCode::FAILURE
        }
    }
}
