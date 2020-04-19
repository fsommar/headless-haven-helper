use serde::de::{DeserializeSeed, IntoDeserializer, Visitor};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub struct State {
    round: i32,
    scenario_number: i32,
    scenario_level: i32,
    track_standees: bool,
    ability_cards: bool,
    random_standees: bool,
    elites_first: bool,
    expire_conditions: bool,
    solo: bool,
    hide_stats: bool,
    calculate_stats: bool,
    can_draw: bool,
    needs_shuffle: bool,
    player_init: i32,
    attack_modifiers: Vec<AttackModifier>,
    attack_modifiers_discard: Vec<AttackModifier>,
    fire: ElementState,
    ice: ElementState,
    air: ElementState,
    earth: ElementState,
    light: ElementState,
    dark: ElementState,
    removed_abilities: Vec<i32>,
    bad_omen: i32,
    ability_decks: Vec<AbilityDeck>,
    actors: Vec<Actor>,
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum AttackModifier {
    Zero = 0,
    Plus1 = 1,
    Plus2 = 2,
    Minus1 = 3,
    Minus2 = 4,
    Miss = 5,
    Crit = 6,
    Bless = 7,
    Curse = 8,
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ElementState {
    Inert = 0,
    Strong = 1,
    Waning = 2,
}

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub struct Ability {
    value: i32,
}

#[derive(Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub struct AbilityDeck {
    id: i32,
    shuffle: bool,
    #[serde(deserialize_with = "deserialize_into_ability")]
    shown_ability: Option<Ability>,
    abilities: Vec<i32>,
    abilities_discard: Vec<i32>,
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum MonsterType {
    Normal = 0,
    Elite = 1,
    Summon = 3,
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum Condition {
    Summoned = 2,
    Stunned = 3,
    Immobilized = 4,
    Disarmed = 5,
    Wounded = 6,
    Muddled = 7,
    Poisoned = 8,
    Strengthened = 9,
    Invisible = 10,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub struct MonsterInstance {
    number: i32,
    tpe: MonsterType,
    // TODO: if MonsterType::Summon, then populate more fields
    is_new: bool,
    hp: i32,
    hp_max: i32,
    conditions: Vec<Condition>,
    conditions_expired: Vec<Condition>,
    conditions_current_turn: Vec<Condition>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub struct Player {
    name: String,
    character_class: CharacterClass,
    xp: i32,
    hp: i32,
    hp_max: i32,
    level: i32,
    loot: i32,
    initiative: i32,
    conditions: Vec<Condition>,
    conditions_expired: Vec<Condition>,
    conditions_current_turn: Vec<Condition>,
    exhausted: bool,
    turn_completed: bool,
    instances: Vec<MonsterInstance>,
}

#[derive(Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub struct Monster {
    id: i32,
    level: i32,
    is_normal: bool,
    is_elite: bool,
    ability: Ability,
    turn_completed: bool,
    instances: Vec<MonsterInstance>,
}

#[derive(Deserialize, Clone, Debug, Eq, PartialOrd, PartialEq)]
pub enum Actor {
    Monster(Monster),
    Player(Player),
}

#[derive(Serialize_repr, Deserialize_repr, Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum CharacterClass {
    Escort = 0,
    Objective = 1,
    Brute = 2,
    Cragheart = 3,
    Mindthief = 4,
    Scoundrel = 5,
    Spellweaver = 6,
    Tinkerer = 7,
    Diviner = 8,
    TwoMinis = 9,
    Lightning = 10,
    AngryFace = 11,
    Triangles = 12,
    Moon = 13,
    CthuluFace = 14,
    TripleArrow = 15,
    Saw = 16,
    MusicNote = 17,
    Circles = 18,
    Sun = 19,
}

pub fn read_varint(buf: &[u8]) -> Option<(usize, i32)> {
    let mut res: i32 = 0;
    for (i, b) in buf.iter().take(5).enumerate() {
        res |= ((b & 0x7F) as i32) << (i * 7) as i32;
        if (b & 0x80) == 0 {
            return Some(dbg!((i + 1, res)));
        }
    }
    None
}

struct Deserializer<'de> {
    input: &'de [u8],
    pos: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_bytes(input: &'de [u8]) -> Self {
        Deserializer { input, pos: 0 }
    }

    fn parse_varint(&mut self) -> Result<i32, Error> {
        let (len, val) = read_varint(&self.input[self.pos..]).ok_or(Error::Empty)?;
        self.pos += len;
        Ok(val)
    }

    fn read_byte(&mut self) -> u8 {
        let res = self.input[self.pos];
        self.pos += 1;
        res
    }

    fn parse_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_byte() == 1)
    }

    fn parse_str(&mut self) -> Result<&'de str, Error> {
        dbg!("parse_str");
        let b = self.read_byte();
        if (b & 0x80) == 0 {
            // ASCII for some reason
            return Ok("");
        }
        let mut len = (b & 0x3F) as u32;
        if (b & 0x40) != 0 {
            //            len = std::iter::from_fn(|| Some(self.read_byte()))
            //                .take_while(|b| (b & 0x80) != 0)
            //                .take(4)
            //                .enumerate()
            //                .fold(len, |acc, (i, b)| acc | (b & 0x7F) << (7 * i as u8 - 1));
            let b = self.read_byte() as u32;
            len |= (b & 0x7F) << 6;
            if (b & 0x80) != 0 {
                let b = self.read_byte() as u32;
                len |= (b & 0x7F) << 13;
                if (b & 0x80) != 0 {
                    let b = self.read_byte() as u32;
                    len |= (b & 0x7F) << 20;
                    if (b & 0x80) != 0 {
                        let b = self.read_byte() as u32;
                        len |= (b & 0x7F) << 27;
                    }
                }
            }
        }
        if len <= 1 {
            Ok("")
        } else {
            std::str::from_utf8(&self.input[self.pos - len as usize + 1..self.pos])
                .map_err(|err| serde::de::Error::custom(err.to_string()))
        }
    }
}

pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_bytes(bytes);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

#[derive(Debug)]
pub enum Error {
    Empty,
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(desc: T) -> Error {
        // ErrorKind::Custom(desc.to_string()).into()
        dbg!(desc.to_string());
        Error::Empty
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::Empty => "empty",
            }
        )
    }
}

impl<'de, 'a> serde::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_varint()?)
    }

    fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_varint()? as u8)
    }

    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(dbg!(self.parse_str())?)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len: i32 = serde::Deserialize::deserialize(&mut *self)?;

        self.deserialize_tuple(len as usize, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct Access<'a, 'de> {
            deserializer: &'a mut Deserializer<'de>,
            len: usize,
        }

        impl<'a, 'de> serde::de::SeqAccess<'de> for Access<'a, 'de> {
            type Error = Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
            where
                T: serde::de::DeserializeSeed<'de>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value =
                        serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        visitor.visit_seq(Access {
            deserializer: self,
            len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct Enum<'a, 'de> {
            deserializer: &'a mut Deserializer<'de>,
        }

        impl<'de, 'a> serde::de::EnumAccess<'de> for Enum<'a, 'de> {
            type Error = Error;
            type Variant = Self;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                let variant = self.deserializer.read_byte();
                dbg!(variant);
                let val = seed.deserialize(variant.into_deserializer())?;
                Ok((val, self))
            }
        }

        impl<'de, 'a> serde::de::VariantAccess<'de> for Enum<'a, 'de> {
            type Error = Error;

            fn unit_variant(self) -> Result<(), Error> {
                Err(serde::de::Error::custom("unexpected"))
            }

            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
            where
                T: DeserializeSeed<'de>,
            {
                seed.deserialize(self.deserializer)
            }

            // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
            // deserialize the sequence of data here.
            fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                serde::de::Deserializer::deserialize_tuple(self.deserializer, len, visitor)
            }

            fn struct_variant<V>(
                self,
                fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, Error>
            where
                V: Visitor<'de>,
            {
                serde::de::Deserializer::deserialize_tuple(self.deserializer, fields.len(), visitor)
            }
        }

        visitor.visit_enum(Enum { deserializer: self })
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::custom("identifiers are not supported"))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

fn deserialize_into_ability<'de, D>(deserializer: D) -> Result<Option<Ability>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct AbilityVisitor;

    impl<'de> Visitor<'de> for AbilityVisitor {
        type Value = Ability;

        fn expecting<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> std::fmt::Result {
            f.write_str("a varint")
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Ability {
                value: if value == 0 { 0 } else { value - 1 },
            })
        }
    }

    impl<'de> Deserialize<'de> for Ability {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_i32(AbilityVisitor)
        }
    }

    let v = deserializer.deserialize_i32(AbilityVisitor)?;
    Ok(Some(v).filter(|a| a.value != 0))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn state() {
        let bytes = include_bytes!("example_state.bin");
        let state: State = from_bytes(bytes).unwrap();
        println!("{:#?}", state);
        assert_eq!(
            state,
            State {
                round: 5,
                scenario_number: 5,
                scenario_level: 1,
                track_standees: true,
                ability_cards: true,
                random_standees: true,
                elites_first: true,
                expire_conditions: true,
                solo: false,
                hide_stats: false,
                calculate_stats: true,
                can_draw: false,
                needs_shuffle: false,
                player_init: 3,
                attack_modifiers: vec![
                    AttackModifier::Minus1,
                    AttackModifier::Crit,
                    AttackModifier::Plus1,
                    AttackModifier::Zero,
                    AttackModifier::Minus2,
                    AttackModifier::Zero,
                    AttackModifier::Miss,
                    AttackModifier::Plus1,
                    AttackModifier::Minus1,
                    AttackModifier::Plus2,
                    AttackModifier::Minus1,
                    AttackModifier::Zero,
                    AttackModifier::Plus1,
                    AttackModifier::Zero,
                    AttackModifier::Zero,
                    AttackModifier::Plus1,
                    AttackModifier::Minus1,
                    AttackModifier::Zero,
                    AttackModifier::Minus1,
                ],
                attack_modifiers_discard: vec![AttackModifier::Plus1,],
                fire: ElementState::Inert,
                ice: ElementState::Inert,
                air: ElementState::Inert,
                earth: ElementState::Inert,
                light: ElementState::Strong,
                dark: ElementState::Inert,
                removed_abilities: vec![],
                bad_omen: 0,
                ability_decks: vec![
                    AbilityDeck {
                        id: 4,
                        shuffle: false,
                        shown_ability: Some(Ability { value: 39 },),
                        abilities: vec![32, 38, 33, 36, 34, 37,],
                        abilities_discard: vec![35, 39,],
                    },
                    AbilityDeck {
                        id: 7,
                        shuffle: false,
                        shown_ability: None,
                        abilities: vec![58, 60, 59, 56, 63, 57, 61, 62,],
                        abilities_discard: vec![],
                    },
                    AbilityDeck {
                        id: 8,
                        shuffle: false,
                        shown_ability: None,
                        abilities: vec![66, 70, 68, 69, 71, 64, 65, 67,],
                        abilities_discard: vec![],
                    },
                    AbilityDeck {
                        id: 14,
                        shuffle: false,
                        shown_ability: Some(Ability { value: 117 },),
                        abilities: vec![116, 112, 115, 113, 119,],
                        abilities_discard: vec![118, 114, 117,],
                    },
                    AbilityDeck {
                        id: 18,
                        shuffle: false,
                        shown_ability: None,
                        abilities: vec![144, 145, 150, 148, 149, 146, 151, 147,],
                        abilities_discard: vec![],
                    },
                ],
                actors: vec![
                    Actor::Player(Player {
                        name: String::new(),
                        character_class: CharacterClass::Spellweaver,
                        xp: 1,
                        hp: 6,
                        hp_max: 6,
                        level: 1,
                        loot: 0,
                        initiative: 12,
                        conditions: vec![],
                        conditions_expired: vec![],
                        conditions_current_turn: vec![],
                        exhausted: false,
                        turn_completed: false,
                        instances: vec![],
                    },),
                    Actor::Player(Player {
                        name: String::new(),
                        character_class: CharacterClass::Scoundrel,
                        xp: 0,
                        hp: 8,
                        hp_max: 8,
                        level: 1,
                        loot: 0,
                        initiative: 12,
                        conditions: vec![],
                        conditions_expired: vec![],
                        conditions_current_turn: vec![],
                        exhausted: false,
                        turn_completed: false,
                        instances: vec![],
                    },),
                    Actor::Player(Player {
                        name: String::new(),
                        character_class: CharacterClass::Brute,
                        xp: 0,
                        hp: 10,
                        hp_max: 10,
                        level: 1,
                        loot: 0,
                        initiative: 12,
                        conditions: vec![],
                        conditions_expired: vec![],
                        conditions_current_turn: vec![],
                        exhausted: false,
                        turn_completed: false,
                        instances: vec![],
                    },),
                    Actor::Monster(Monster {
                        id: 7,
                        level: 1,
                        is_normal: false,
                        is_elite: false,
                        ability: Ability { value: 39 },
                        turn_completed: false,
                        instances: vec![
                            MonsterInstance {
                                number: 1,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 2,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 3,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 4,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 5,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 6,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                        ],
                    },),
                    Actor::Player(Player {
                        name: String::new(),
                        character_class: CharacterClass::Mindthief,
                        xp: 0,
                        hp: 6,
                        hp_max: 6,
                        level: 1,
                        loot: 0,
                        initiative: 55,
                        conditions: vec![],
                        conditions_expired: vec![],
                        conditions_current_turn: vec![],
                        exhausted: false,
                        turn_completed: false,
                        instances: vec![],
                    },),
                    Actor::Monster(Monster {
                        id: 19,
                        level: 1,
                        is_normal: false,
                        is_elite: false,
                        ability: Ability { value: 117 },
                        turn_completed: false,
                        instances: vec![
                            MonsterInstance {
                                number: 1,
                                tpe: MonsterType::Elite,
                                is_new: false,
                                hp: 6,
                                hp_max: 6,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 7,
                                tpe: MonsterType::Elite,
                                is_new: false,
                                hp: 6,
                                hp_max: 6,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                            MonsterInstance {
                                number: 10,
                                tpe: MonsterType::Normal,
                                is_new: false,
                                hp: 5,
                                hp_max: 5,
                                conditions: vec![],
                                conditions_expired: vec![],
                                conditions_current_turn: vec![],
                            },
                        ],
                    },),
                    Actor::Monster(Monster {
                        id: 24,
                        level: 1,
                        is_normal: false,
                        is_elite: false,
                        ability: Ability { value: 0 },
                        turn_completed: false,
                        instances: vec![],
                    },),
                    Actor::Monster(Monster {
                        id: 10,
                        level: 1,
                        is_normal: false,
                        is_elite: false,
                        ability: Ability { value: 0 },
                        turn_completed: false,
                        instances: vec![],
                    },),
                    Actor::Monster(Monster {
                        id: 11,
                        level: 1,
                        is_normal: false,
                        is_elite: false,
                        ability: Ability { value: 0 },
                        turn_completed: false,
                        instances: vec![],
                    },),
                ],
            }
        );
    }

    #[test]
    fn player() {
        let x: Player = from_bytes(&[
            0x0, /* name */
            0x2, /* class */
            0xE, 0xE, 0x8, 0x2, 0x1, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0,
        ])
        .unwrap();
        println!("player {:#?}", x);
        assert_eq!(
            x,
            Player {
                name: String::new(),
                character_class: CharacterClass::Brute,
                xp: 14,
                hp: 14,
                hp_max: 8,
                level: 2,
                loot: 1,
                initiative: 0,
                conditions: Vec::new(),
                conditions_expired: Vec::new(),
                conditions_current_turn: Vec::new(),
                exhausted: true,
                turn_completed: false,
                instances: Vec::new(),
            }
        );
    }

    #[test]
    fn monster() {
        let x: Monster = from_bytes(&[
            0x3, /* id */
            0xA, /* level */
            0x1, /* is_normal */
            0x0, /* is_elite */
            0x20, 0x1, 0x0,
        ])
        .unwrap();
        println!("monster {:#?}", x);
        assert_eq!(
            x,
            Monster {
                id: 3,
                level: 10,
                is_normal: true,
                is_elite: false,
                ability: Ability { value: 31 },
                turn_completed: true,
                instances: Vec::new(),
            }
        );
    }

    #[test]
    fn actor() {
        let x: Actor = from_bytes(&[
            0x1, /* enum indicator */
            0x0, /* name */
            0x2, /* class */
            0xE, 0xE, 0x8, 0x2, 0x1, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0,
        ])
        .unwrap();
        println!("actor {:#?}", x);
        assert_eq!(
            x,
            Actor::Player(Player {
                name: String::new(),
                character_class: CharacterClass::Brute,
                xp: 14,
                hp: 14,
                hp_max: 8,
                level: 2,
                loot: 1,
                initiative: 0,
                conditions: Vec::new(),
                conditions_expired: Vec::new(),
                conditions_current_turn: Vec::new(),
                exhausted: true,
                turn_completed: false,
                instances: Vec::new(),
            })
        );

        let x: Actor = from_bytes(&[
            0x0, /* enum indicator */
            0x3, /* id */
            0xA, /* level */
            0x1, /* is_normal */
            0x0, /* is_elite */
            0x20, 0x1, 0x0,
        ])
        .unwrap();
        println!("actor {:#?}", x);
        assert_eq!(
            x,
            Actor::Monster(Monster {
                id: 3,
                level: 10,
                is_normal: true,
                is_elite: false,
                ability: Ability { value: 31 },
                turn_completed: true,
                instances: Vec::new(),
            })
        );
    }
}
