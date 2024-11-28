use anyhow::{anyhow, Result};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt::Display;
use strum::VariantArray;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new_shuffled() -> Self {
        let mut cards = Vec::with_capacity(Suit::VARIANTS.len() * Number::VARIANTS.len());
        for suit in Suit::VARIANTS.iter() {
            for number in Number::VARIANTS.iter() {
                cards.push(Card::new(*number, *suit))
            }
        }
        cards.shuffle(&mut thread_rng());

        Self { cards }
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    pub fn cards(&self) -> impl ExactSizeIterator<Item = &Card> {
        self.cards.iter()
    }

    pub fn remove(&mut self, to_remove: &[Card]) {
        self.cards.retain(|card| !to_remove.contains(card))
    }

    pub fn draw_hand(&mut self) -> Result<Hand> {
        let len = self.cards.len();
        Hand::from_slice(self.cards.drain(len - 4..).as_slice())
            .map_err(|_| anyhow!("expected 4+ cards in the deck"))
    }

    pub fn draw(&mut self) -> Card {
        self.cards.pop().unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, VariantArray)]
pub enum Suit {
    H,
    D,
    S,
    C,
}

// Card value represented as an enum (to avoid bound checks, hopefully)
#[derive(Clone, Copy, PartialEq, Eq, Debug, VariantArray)]
pub enum Number {
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
pub struct Card {
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
pub struct Hand {
    cards: [Card; 4],
}

impl Hand {
    pub fn from_array(cards: [Card; 4]) -> Self {
        Self { cards }
    }

    pub fn from_slice(slice: &[Card]) -> Result<Self> {
        Ok(Self {
            cards: slice
                .try_into()
                .map_err(|_| anyhow!("4 card expected, {} given", slice.len()))?,
        })
    }

    pub fn score(&self, starter: Card, crib: bool) -> u8 {
        let cards4 = &self.cards;
        let mut cards5: [Card; 5] = [cards4[0], cards4[1], cards4[2], cards4[3], starter];
        cards5.sort();

        self.score_suit(starter, crib)
            + self.score_fifteens(&cards5)
            + self.score_pairs(&cards5)
            + self.score_straights(&cards5)
            + self.score_knob(starter)
    }

    fn score_suit(&self, starter: Card, crib: bool) -> u8 {
        let same_suit = self.cards[1..].iter().all(|c| c.suit == self.cards[0].suit);

        if same_suit {
            if self.cards[0].suit == starter.suit {
                5
            } else if !crib {
                4
            } else {
                0
            }
        } else {
            0
        }
    }

    fn score_fifteens(&self, cards5: &[Card; 5]) -> u8 {
        let mut fifteens = 0;

        for i in 0..cards5.len() {
            for j in i + 1..cards5.len() {
                if cards5[i].value() + cards5[j].value() == 15 {
                    fifteens += 1
                }
                for k in j + 1..cards5.len() {
                    // triple
                    if cards5[i].value() + cards5[j].value() + cards5[k].value() == 15 {
                        fifteens += 1
                    }
                    for l in k + 1..cards5.len() {
                        // 4 cards
                        if cards5[i].value()
                            + cards5[j].value()
                            + cards5[k].value()
                            + cards5[l].value()
                            == 15
                        {
                            fifteens += 1;
                        }
                    }
                }
            }
        }

        if cards5.iter().map(Card::value).sum::<u8>() == 15 {
            fifteens += 1;
        }

        fifteens * 2
    }

    fn score_pairs(&self, cards5: &[Card; 5]) -> u8 {
        let mut pairs = 0u8;

        for i in 0..cards5.len() {
            for j in i + 1..cards5.len() {
                if cards5[i].number == cards5[j].number {
                    pairs += 1
                }
            }
        }

        pairs * 2
    }

    fn score_straights(&self, cards5: &[Card; 5]) -> u8 {
        let mut range = cards5[0].number as usize..cards5[0].number as usize;
        for (c1, c2) in cards5.iter().copied().tuple_windows() {
            let new_end = c2.number as usize;

            if c1.number as u8 + 1 >= c2.number as u8 {
                range.end = new_end;
            } else if range.end - range.start >= 2 {
                break;
            } else {
                range = new_end..new_end
            }
        }

        let straight_size = (range.end - range.start) as u8 + 1;
        if straight_size >= 3 {
            let mut count_by_numbers = [0u8; 13];
            for card in cards5.iter() {
                count_by_numbers[card.number as usize] += 1;
            }

            straight_size
                * count_by_numbers[range.start..=range.end]
                    .iter()
                    .copied()
                    .fold(1, |memo, count| memo * count)
        } else {
            0
        }
    }

    pub fn score_knob(&self, starter: Card) -> u8 {
        let knob = Card::new(Number::J, starter.suit);
        self.cards.iter().contains(&knob) as u8
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = self.cards;
        write!(f, "{} {} {} {}", c[0], c[1], c[2], c[3])
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

pub fn parse_cards(input: &str) -> Result<Vec<Card>> {
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
        fn value_for(number: Number) -> u8 {
            Card::new(number, Suit::C).value()
        }

        assert_eq!(1, value_for(Number::A));
        assert_eq!(2, value_for(Number::C2));
        assert_eq!(3, value_for(Number::C3));
        assert_eq!(4, value_for(Number::C4));
        assert_eq!(5, value_for(Number::C5));
        assert_eq!(6, value_for(Number::C6));
        assert_eq!(7, value_for(Number::C7));
        assert_eq!(8, value_for(Number::C8));
        assert_eq!(9, value_for(Number::C9));
        assert_eq!(10, value_for(Number::T));
        assert_eq!(10, value_for(Number::J));
        assert_eq!(10, value_for(Number::Q));
        assert_eq!(10, value_for(Number::K));
    }

    #[test]
    fn score_tests() -> Result<()> {
        // Fifteen 6
        assert_eq!(6, score_hand("2d Js Ks 5h", "Th")?);
        // Fifteen with 3 cards
        assert_eq!(2, score_hand("1d 2s 6s 8h", "Th")?);
        // Fifteen with 4 cards (+1 pair)
        assert_eq!(4, score_hand("1d 1s 3d 5h ", "8h")?);
        // Fifteen with 5 cards (+ 5 for straight)
        assert_eq!(7, score_hand("1d 2s 3s 4h", "5h")?);

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

        // 3 straight
        assert_eq!(3, score_hand("1s Th Jd Qc", "8h")?);

        // 4 straight
        assert_eq!(4, score_hand("1s Th Jd Qc", "Kh")?);

        // 5 straight
        assert_eq!(5, score_hand("9s Th Jd Qc", "Kh")?);

        // 2x 3 straight
        assert_eq!(8, score_hand("1s Th Jd Qc", "Qh")?);

        // 9x 3 straight
        assert_eq!(15, score_hand("Qs Th Jd Qc", "Qh")?);

        // 4x 3 straight
        assert_eq!(16, score_hand("Th Ts Jd Qc", "Qh")?);

        // Not-straight of 2, starting from 2 (tests for improperly starting straight at Ace (0))
        assert_eq!(2, score_hand("2h 3s 9d 8c", "Qh")?);

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
