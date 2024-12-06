use std::io::{stdout, Write};

use anyhow::Result;
use itertools::Itertools;

use crible_core::*;

struct Scores {
    scores: [(Card, u8); 52 - 4],
    len: u8,
    sorted: bool,
}

impl Scores {
    fn new() -> Self {
        Scores {
            scores: [(Card::new(Number::A, Suit::H), 0); 52 - 4],
            len: 0,
            sorted: true,
        }
    }

    fn push(&mut self, card: Card, score: u8) {
        self.sorted = false;
        self.scores[self.len as usize] = (card, score);
        self.len += 1;
    }

    fn sort(&mut self) {
        if !self.sorted {
            self.scores[0..self.len as usize].sort_by(|(_, a), (_, b)| b.cmp(a));
            self.sorted = true;
        }
    }

    fn mean(&self) -> f32 {
        self.scores
            .iter()
            .map(|(_, score)| *score as u32)
            .sum::<u32>() as f32
            / self.len as f32
    }

    fn iter(&self) -> impl Iterator<Item = (Card, u8)> + '_ {
        self.scores[0..self.len as usize].iter().copied()
    }
}

fn main() -> Result<()> {
    let input: String = std::env::args().skip(1).collect();

    let mut deck = Deck::new_shuffled();
    let mut cards = parse_cards(input.as_str())?;
    cards.sort();
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

    let top_n = 4;
    for (hand, scores) in results.iter().take(top_n) {
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

    match results.len().saturating_sub(top_n) {
        0 => {} // no-op
        1 => println!("... and 1 worse hand"),
        n => println!("... and {n} worse hands"),
    }

    Ok(())
}
