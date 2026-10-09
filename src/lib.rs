use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Quiz {
    pub categories: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub subcategories: Vec<Subcategory>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subcategory {
    pub name: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerMode {
    Single,
    Multiple,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub text: String,
    pub mode: AnswerMode,
    pub answers: Vec<String>,
    /// Zero-based indices into `answers`.
    pub correct: Vec<usize>,
}

impl Question {
    pub fn validate(&self) -> Result<(), String> {
        if self.text.trim().is_empty() {
            return Err("Tekst pitanja ne sme biti prazan.".into());
        }
        if self.answers.len() < 2 || self.answers.iter().any(|a| a.trim().is_empty()) {
            return Err("Potrebna su bar dva neprazna ponuđena odgovora.".into());
        }
        for (i, answer) in self.answers.iter().enumerate() {
            if self.answers[..i].iter().any(|a| same_name(a, answer)) {
                return Err("Ponuđeni odgovori moraju biti različiti.".into());
            }
        }
        validate_selection(&self.correct, self.answers.len(), self.mode)
    }

    pub fn is_correct(&self, selected: &[usize]) -> bool {
        validate_selection(selected, self.answers.len(), self.mode).is_ok()
            && selected.len() == self.correct.len()
            && selected.iter().all(|i| self.correct.contains(i))
    }
}

pub fn same_name(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

fn validate_selection(indices: &[usize], count: usize, mode: AnswerMode) -> Result<(), String> {
    if indices.is_empty() || indices.iter().any(|&i| i >= count) {
        return Err(format!("Izaberite brojeve odgovora od 1 do {count}."));
    }
    for (i, value) in indices.iter().enumerate() {
        if indices[..i].contains(value) {
            return Err("Isti broj odgovora ne može biti unet više puta.".into());
        }
    }
    match mode {
        AnswerMode::Single if indices.len() != 1 => {
            Err("Za ovaj tip pitanja izaberite tačno jedan odgovor.".into())
        }
        AnswerMode::Multiple if indices.len() < 2 => {
            Err("Za ovaj tip pitanja izaberite bar dva odgovora.".into())
        }
        _ => Ok(()),
    }
}

pub fn parse_selection(input: &str, count: usize, mode: AnswerMode) -> Result<Vec<usize>, String> {
    let indices = input
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(1))
                .ok_or_else(|| "Unesite brojeve odgovora, npr. 1 ili 1,3.".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_selection(&indices, count, mode)?;
    Ok(indices)
}

impl Quiz {
    pub fn load(path: &Path) -> io::Result<Self> {
        let data = match fs::read(path) {
            Ok(data) => data,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(e) => return Err(e),
        };
        let quiz: Self = serde_json::from_slice(&data).map_err(io::Error::other)?;
        quiz.validate().map_err(io::Error::other)?;
        Ok(quiz)
    }

    fn validate(&self) -> Result<(), String> {
        for (i, category) in self.categories.iter().enumerate() {
            if category.name.trim().is_empty()
                || self.categories[..i]
                    .iter()
                    .any(|c| same_name(&c.name, &category.name))
            {
                return Err("Prazan ili ponovljen naziv kategorije.".into());
            }
            for (j, subcategory) in category.subcategories.iter().enumerate() {
                if subcategory.name.trim().is_empty()
                    || category.subcategories[..j]
                        .iter()
                        .any(|s| same_name(&s.name, &subcategory.name))
                {
                    return Err("Prazan ili ponovljen naziv potkategorije.".into());
                }
                for question in &subcategory.questions {
                    question.validate()?;
                }
            }
        }
        Ok(())
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        self.validate().map_err(io::Error::other)?;
        let data = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, data)?;
        fs::rename(temporary, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_selection_and_answer_modes() {
        assert_eq!(
            parse_selection("3, 1", 3, AnswerMode::Multiple).unwrap(),
            vec![2, 0]
        );
        for input in ["", "0", "4", "abc", "1,1", "1,2"] {
            assert!(
                parse_selection(input, 3, AnswerMode::Single).is_err(),
                "{input}"
            );
        }
        assert!(parse_selection("1", 3, AnswerMode::Multiple).is_err());
        assert!(parse_selection("1,1", 3, AnswerMode::Multiple).is_err());
    }

    #[test]
    fn grading_requires_exact_set() {
        let q = Question {
            text: "Prosti brojevi?".into(),
            mode: AnswerMode::Multiple,
            answers: vec!["2".into(), "3".into(), "4".into()],
            correct: vec![0, 1],
        };
        assert!(q.validate().is_ok());
        assert!(q.is_correct(&[1, 0]));
        for selection in [vec![0], vec![0, 2], vec![0, 1, 2], vec![0, 0]] {
            assert!(!q.is_correct(&selection));
        }
    }

    #[test]
    fn rejects_duplicate_answers_and_invalid_correct_indices() {
        let mut q = Question {
            text: "Pitanje".into(),
            mode: AnswerMode::Single,
            answers: vec!["Da".into(), " da ".into()],
            correct: vec![0],
        };
        assert!(q.validate().is_err());
        q.answers[1] = "Ne".into();
        q.correct = vec![2];
        assert!(q.validate().is_err());
    }

    #[test]
    fn storage_roundtrip_and_corrupt_file_protection() {
        let dir = std::env::temp_dir().join(format!("quiz-maker-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("quiz.json");
        assert!(Quiz::load(&path).unwrap().categories.is_empty());
        let quiz = Quiz {
            categories: vec![Category {
                name: "Nauka".into(),
                subcategories: vec![Subcategory {
                    name: "Matematika".into(),
                    questions: vec![Question {
                        text: "2 + 2?".into(),
                        mode: AnswerMode::Single,
                        answers: vec!["3".into(), "4".into()],
                        correct: vec![1],
                    }],
                }],
            }],
        };
        quiz.save(&path).unwrap();
        let loaded = Quiz::load(&path).unwrap();
        assert!(loaded.categories[0].subcategories[0].questions[0].is_correct(&[1]));
        quiz.save(&path).unwrap();
        fs::write(&path, "nije JSON").unwrap();
        assert!(Quiz::load(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "nije JSON");
        fs::remove_dir_all(dir).unwrap();
    }
}
