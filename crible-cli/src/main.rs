use std::io::{stdout, Write};

use anyhow::Result;
use itertools::Itertools;

use crible_core::*;

struct Scores {
    scores: [Option<(Card, u8)>; 52 - 4],
    len: u8,
    sorted: bool,
}

impl Scores {
    fn new() -> Self {
        Scores {
            scores: [None; 52 - 4],
            len: 0,
            sorted: true,
        }
    }

    fn push(&mut self, card: Card, score: u8) {
        self.sorted = false;
        self.scores[self.len as usize] = Some((card, score));
        self.len += 1;
    }

    fn sort(&mut self) {
        if !self.sorted {
            self.scores.sort_by(|a, b| match (a, b) {
                (Some((_, a)), Some((_, b))) => b.cmp(a),
                _ => b.cmp(a),
            });
            self.sorted = true;
        }
    }

    fn mean(&self) -> f32 {
        self.scores
            .iter()
            .map(|entry| entry.map(|(_, score)| score).unwrap_or(0) as u32)
            .sum::<u32>() as f32
            / self.len as f32
    }

    fn iter(&self) -> impl Iterator<Item = (Card, u8)> + '_ {
        self.scores[0..self.len as usize].iter().map(|o| o.unwrap())
    }
}

fn main() -> Result<()> {
    let mut deck = Deck::new_shuffled();

    let input = std::env::var("HAND").expect("Can't parsed HAND env var");
    let cards = parse_cards(input.as_str())?;
    deck.remove(&cards);

    let mut results: Vec<(Hand, Scores)> = Vec::new();

    // All possible combinaisons of 4 cards
    for (c1, c2, c3, c4) in cards.iter().copied().tuple_combinations() {
        let hand = Hand::from_array([c1, c2, c3, c4]);

        let mut scores = Scores::new();

        for starter in deck.cards().copied() {
            let score = hand.score(starter, false);
            scores.push(starter, score);
        }
        scores.sort();

        results.push((hand, scores))
    }

    results.sort_by(|(_, a), (_, b)| b.mean().partial_cmp(&a.mean()).unwrap());

    let mut lock = stdout().lock();
    writeln!(
        lock,
        "What's the best play for {}?\n",
        cards.iter().join(" ")
    )?;

    for (hand, scores) in results {
        let mut top_starters: Vec<(u8, Vec<Card>)> = Default::default();
        for (score, chunks) in &scores.iter().chunk_by(|(_, score)| *score) {
            let mut starters = chunks.map(|(card, _)| card).collect::<Vec<_>>();
            starters.sort();
            top_starters.push((score, starters));
        }

        writeln!(lock, "Hand: {hand}  Mean: {:.2}", scores.mean())?;
        writeln!(lock, "  Top starters: ")?;
        for (score, starters) in top_starters {
            write!(lock, "      {: >2} points: ", score)?;
            write!(lock, "{}", starters.iter().take(10).join(" "))?;
            if starters.len() > 10 {
                write!(lock, " ...")?;
            }
            write!(lock, "\n")?;
        }
        writeln!(lock, "")?;
    }
    Ok(())
}
