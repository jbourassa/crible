use anyhow::Result;
use itertools::Itertools;

use crible_core::*;

fn main() -> Result<()> {
    let mut deck = Deck::new_shuffled();

    let cards = parse_cards("5d Td Qd Jd")?;
    deck.remove(&cards);

    let mut results: Vec<(Hand, [Option<(Card, u8)>; 48])> = Vec::new();

    // All possible combinaisons of 4 cards
    for (c1, c2, c3, c4) in cards.iter().copied().tuple_combinations() {
        let hand = Hand::from_array([c1, c2, c3, c4]);

        let mut entries: [Option<(Card, u8)>; 48] = [None; 48];

        for (i, starter) in deck.cards().copied().enumerate() {
            let score = hand.score(starter, false);
            entries[i] = Some((starter, score));
        }

        results.push((hand, entries))
    }

    Ok(())
}
