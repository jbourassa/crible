use anyhow::{anyhow, Result};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::BTreeSet;
use std::fmt::Display;

struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new_shuffled() -> Self {
        let mut cards = Vec::with_capacity(Suit::all().len() * Number::all().len());
        for suit in Suit::all() {
            for number in Number::all() {
                cards.push(Card { suit, number })
            }
        }
        cards.shuffle(&mut thread_rng());

        Self { cards }
    }

    fn remove(&mut self, to_remove: &[Card]) {
        self.cards.retain(|card| !to_remove.contains(card))
    }

    fn draw(&mut self) -> Card {
        self.cards.pop().unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Suit {
    H,
    D,
    S,
    C,
}

impl Suit {
    pub fn all() -> impl ExactSizeIterator<Item = Suit> {
        const SUITS: &[Suit] = &[Suit::H, Suit::D, Suit::S, Suit::C];
        SUITS.iter().copied()
    }
}

// Card value represented as an enum (to avoid bound checks, hopefully)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Number {
    A,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    T,
    J,
    Q,
    K,
}

impl Number {
    pub fn all() -> impl ExactSizeIterator<Item = Number> {
        const NUMBERS: &[Number] = &[
            Number::A,
            Number::C2,
            Number::C3,
            Number::C4,
            Number::C5,
            Number::C6,
            Number::C7,
            Number::C8,
            Number::C9,
            Number::T,
            Number::J,
            Number::Q,
            Number::K,
        ];
        NUMBERS.iter().copied()
    }

    /// Return the numerical value a value between 1 and 10
    pub fn value(&self) -> u8 {
        const VALUES: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10];
        VALUES[*self as usize]
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::A => write!(f, "A"),
            Number::C2 => write!(f, "2"),
            Number::C3 => write!(f, "3"),
            Number::C4 => write!(f, "4"),
            Number::C5 => write!(f, "5"),
            Number::C6 => write!(f, "6"),
            Number::C7 => write!(f, "7"),
            Number::C8 => write!(f, "8"),
            Number::C9 => write!(f, "9"),
            Number::T => write!(f, "T"),
            Number::J => write!(f, "J"),
            Number::Q => write!(f, "Q"),
            Number::K => write!(f, "K"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Card {
    number: Number,
    suit: Suit,
}

impl Card {
    pub fn new(number: Number, suit: Suit) -> Self {
        Self { number, suit }
    }

    pub fn value(&self) -> u8 {
        self.number.value()
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.number, self.suit)
    }
}

impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.number as u8).cmp(&(other.number as u8)) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => (self.suit as u8).cmp(&(other.suit as u8)),
        }
    }
}

#[derive(PartialEq, Eq)]
struct Hand {
    cards: [Card; 4],
}

impl Hand {
    fn score(&self, starter: Card, crib: bool) -> u8 {
        let cards4 = &self.cards;
        let mut cards5: [Card; 5] = [cards4[0], cards4[1], cards4[2], cards4[3], starter];
        cards5.sort();

        let same_suit = cards4[1..=3].iter().all(|c| c.suit == cards4[0].suit);
        let suit_points: u8 = if same_suit {
            if cards4[0].suit == starter.suit {
                5
            } else if !crib {
                4
            } else {
                0
            }
        } else {
            0
        };

        let fifteens = cards5
            .iter()
            .powerset()
            .filter(|set| set.iter().map(|c| c.value()).sum::<u8>() == 15)
            .count() as u8;

        let pairs = cards5
            .iter()
            .copied()
            .tuple_combinations()
            .filter(|(c1, c2)| c1.number == c2.number)
            .count() as u8;

        let mut numbers = [0u8; 13];
        for card in cards5.iter() {
            numbers[card.number as usize] += 1;
        }

        let mut range = 0..0;
        for (i, _) in numbers
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, count)| *count > 0)
        {
            if i == range.end + 1 {
                range.end = i;
            } else if range.end - range.start >= 2 {
                break;
            } else {
                range = i..i;
            }
        }

        let straight_size = (range.end - range.start) as u8 + 1;
        let straight_score = if straight_size >= 3 {
            straight_size
                * numbers[range.start..=range.end]
                    .iter()
                    .fold(1, |memo, count| memo * *count)
        } else {
            0
        };

        let knob_score = cards4
            .iter()
            .any(|card| card.number == Number::J && card.suit == starter.suit)
            as u8;

        suit_points + fifteens * 2 + pairs * 2 + straight_score + knob_score
    }
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Suit::S => write!(f, "♠"),
            Suit::H => write!(f, "♥"),
            Suit::D => write!(f, "♦"),
            Suit::C => write!(f, "♣"),
        }
    }
}

fn parse_cards(input: &str) -> Result<Vec<Card>> {
    let mut iter = input.chars();
    let mut cards: Vec<Card> = Vec::with_capacity(6);

    loop {
        match iter.next() {
            Some(char) if char.is_whitespace() => continue,
            Some(number_char) => {
                let number: Number = number_char.try_into()?;
                let suit = match iter.next() {
                    Some(suit) => suit.try_into()?,
                    None => {
                        return Err(anyhow!(
                            "unexpected end, missing suit after {}",
                            number_char
                        ))
                    }
                };

                cards.push(Card::new(number, suit))
            }
            None => break, // we're done!
        };
    }

    Ok(cards)
}

impl TryInto<Number> for char {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Number, Self::Error> {
        match self.to_ascii_uppercase() {
            'A' => Ok(Number::A),
            '1' => Ok(Number::A),
            '2' => Ok(Number::C2),
            '3' => Ok(Number::C3),
            '4' => Ok(Number::C4),
            '5' => Ok(Number::C5),
            '6' => Ok(Number::C6),
            '7' => Ok(Number::C7),
            '8' => Ok(Number::C8),
            '9' => Ok(Number::C9),
            'T' => Ok(Number::T),
            'J' => Ok(Number::J),
            'Q' => Ok(Number::Q),
            'K' => Ok(Number::K),
            _ => Err(anyhow!(
                "Invalid card number: {self}, must be one of: A23456789TJQK"
            )),
        }
    }
}

impl TryInto<Suit> for char {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Suit, Self::Error> {
        match self.to_ascii_lowercase() {
            's' => Ok(Suit::S),
            'h' => Ok(Suit::H),
            'd' => Ok(Suit::D),
            'c' => Ok(Suit::C),
            _ => Err(anyhow!("Invalid suit: {self}, must be one of: shdc")),
        }
    }
}

fn main() -> Result<()> {
    let mut deck = Deck::new_shuffled();
    let s = BTreeSet::<Card>::new();

    let cards = parse_cards("5d Td Qd Jd")?;
    deck.remove(&cards);

    let mut results: Vec<(Hand, [Option<(Card, u8)>; 48])> = Vec::new();

    // All possible combinaisons of 4 cards
    for (c1, c2, c3, c4) in cards.iter().copied().tuple_combinations() {
        let hand = Hand {
            cards: [c1, c2, c3, c4],
        };

        let mut entries: [Option<(Card, u8)>; 48] = [None; 48];

        for (i, starter) in deck.cards.iter().copied().enumerate() {
            let score = hand.score(starter, false);
            entries[i] = Some((starter, score));
        }

        results.push((hand, entries))
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cards_tests() -> Result<()> {
        // Accepts any suits
        assert_eq!(vec![Card::new(Number::A, Suit::H)], parse_cards("Ah")?);
        assert_eq!(vec![Card::new(Number::A, Suit::D)], parse_cards("Ad")?);
        assert_eq!(vec![Card::new(Number::A, Suit::S)], parse_cards("As")?);
        assert_eq!(vec![Card::new(Number::A, Suit::C)], parse_cards("Ac")?);

        // Accepts any casing
        assert_eq!(vec![Card::new(Number::A, Suit::C)], parse_cards("aC")?);

        // Accepts spaces
        assert_eq!(
            vec![Card::new(Number::A, Suit::C), Card::new(Number::A, Suit::H)],
            parse_cards(" Ac  Ah ")?
        );

        // Accepts any of: A123456789TJQK
        assert_eq!(vec![Card::new(Number::A, Suit::C)], parse_cards("Ac")?);
        assert_eq!(vec![Card::new(Number::A, Suit::C)], parse_cards("1c")?);
        assert_eq!(vec![Card::new(Number::C2, Suit::C)], parse_cards("2c")?);
        assert_eq!(vec![Card::new(Number::C3, Suit::C)], parse_cards("3c")?);
        assert_eq!(vec![Card::new(Number::C4, Suit::C)], parse_cards("4c")?);
        assert_eq!(vec![Card::new(Number::C5, Suit::C)], parse_cards("5c")?);
        assert_eq!(vec![Card::new(Number::C6, Suit::C)], parse_cards("6c")?);
        assert_eq!(vec![Card::new(Number::C7, Suit::C)], parse_cards("7c")?);
        assert_eq!(vec![Card::new(Number::C8, Suit::C)], parse_cards("8c")?);
        assert_eq!(vec![Card::new(Number::C9, Suit::C)], parse_cards("9c")?);
        assert_eq!(vec![Card::new(Number::T, Suit::C)], parse_cards("Tc")?);
        assert_eq!(vec![Card::new(Number::J, Suit::C)], parse_cards("Jc")?);
        assert_eq!(vec![Card::new(Number::Q, Suit::C)], parse_cards("Qc")?);
        assert_eq!(vec![Card::new(Number::K, Suit::C)], parse_cards("Kc")?);

        let assert_error = |cards, msg: &str| {
            assert_eq!(
                Err(msg.to_string()),
                parse_cards(cards).map_err(|e| e.to_string()),
            );
        };

        assert_error(
            "Fc",
            "Invalid card number: F, must be one of: A23456789TJQK",
        );
        assert_error("2g", "Invalid suit: g, must be one of: shdc");
        assert_error("2", "unexpected end, missing suit after 2");

        Ok(())
    }

    #[test]
    fn card_value() {
        fn card_value(number: Number) -> u8 {
            Card::new(number, Suit::C).value()
        }

        assert_eq!(1, card_value(Number::A));
        assert_eq!(2, card_value(Number::C2));
        assert_eq!(3, card_value(Number::C3));
        assert_eq!(4, card_value(Number::C4));
        assert_eq!(5, card_value(Number::C5));
        assert_eq!(6, card_value(Number::C6));
        assert_eq!(7, card_value(Number::C7));
        assert_eq!(8, card_value(Number::C8));
        assert_eq!(9, card_value(Number::C9));
        assert_eq!(10, card_value(Number::T));
        assert_eq!(10, card_value(Number::J));
        assert_eq!(10, card_value(Number::Q));
        assert_eq!(10, card_value(Number::K));
    }

    #[test]
    fn score_tests() -> Result<()> {
        /*
        // Fifteen 6
        assert_eq!(6, score_hand("2d Js Ks 5h", "Th")?);

        // Hand all spade
        assert_eq!(4, score_hand("2s 4s Qs Ks", "Th")?);
        // Hand + starter all spade
        assert_eq!(5, score_hand("2s 4s Qs Ks", "Ts")?);
        // Crib all spade: no good
        assert_eq!(0, score_crib("2s 4s Qs Ks", "Th")?);

        // Crib + starter all spade
        assert_eq!(5, score_crib("2s 4s Qs Ks", "Ts")?);

        // Pair(s)
        assert_eq!(2, score_hand("2s 2h 9d Kc", "Qh")?);
        assert_eq!(4, score_hand("2s 2h 9d 9c", "Qh")?);

        // Triple
        assert_eq!(6, score_hand("2s 2h 2d Kc", "Qh")?);

        // 4 of a kind
        assert_eq!(12, score_hand("2s 2h 2d 2c", "3h")?);
        assert_eq!(12, score_hand("2s 2h 2d 3h", "2c")?);
        */

        // 3 straight
        assert_eq!(3, score_hand("1s Th Jd Qc", "8h")?);

        // 4 straight
        assert_eq!(4, score_hand("1s Th Jd Qc", "Kh")?);

        // 2x 3 straight
        assert_eq!(8, score_hand("1s Th Jd Qc", "Qh")?);

        // 4x 3 straight
        assert_eq!(16, score_hand("Th Ts Jd Qc", "Qh")?);

        // Knob
        assert_eq!(1, score_hand("1s 6h 7d Jc", "Qc")?);

        // The one and only
        assert_eq!(29, score_hand("5s 5h 5d Jc", "5c")?);

        Ok(())
    }

    fn hand(input: &str) -> Result<Hand> {
        let cards = parse_cards(input)?;
        Ok(Hand {
            cards: cards.try_into().unwrap(),
        })
    }

    fn score_hand(cards: &str, starter: &str) -> Result<u8> {
        let score = hand(cards)?.score(card(starter)?, false);

        Ok(score)
    }

    fn score_crib(cards: &str, starter: &str) -> Result<u8> {
        let score = hand(cards)?.score(card(starter)?, true);

        Ok(score)
    }

    fn card(input: &str) -> Result<Card> {
        let cards = parse_cards(input)?;
        assert_eq!(1, cards.len());

        Ok(*cards.first().unwrap())
    }
}
