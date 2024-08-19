use anyhow::Context;
use pest::{iterators::Pair, Parser};
use std::str::FromStr;

use crate::{Element, WordXMLDocument};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./xml.pest"]
struct XMLParser;

impl FromStr for WordXMLDocument {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let word_xml = XMLParser::parse(Rule::word_xml, s)
            .context("Failed to parse xml word document.")?
            .next()
            .context("XML document must containt word_xml rule")?;

        let mut xml_document = WordXMLDocument {
            header: Default::default(),
            root: Default::default(),
        };

        for rule in word_xml.into_inner() {
            match rule.as_rule() {
                Rule::header => xml_document.header = rule.as_str().to_string(),
                Rule::element => {
                    xml_document.root = rule.try_into()?;
                }
                otherwise => {
                    return Err(anyhow::Error::msg(format!(
                        "Invalid inner rule while parse xml_document: {:?}",
                        otherwise
                    )))
                }
            }
        }

        Ok(xml_document)
    }
}

impl TryFrom<Pair<'_, Rule>> for super::Element {
    type Error = anyhow::Error;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let mut element = Element::default();
        for rule in value.into_inner() {
            match rule.as_rule() {
                Rule::open_tag => {
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::tag_name => element.name = rule.as_str().to_string(),
                            Rule::attr => element.attrs.push(rule.try_into()?),
                            otherwise => {
                                return Err(anyhow::Error::msg(format!(
                                    "Invalid inner rule while parse open_tag: {:?}",
                                    otherwise
                                )))
                            }
                        }
                    }
                }
                Rule::open_close_tag => {
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::tag_name => element.name = rule.as_str().to_string(),
                            Rule::attr => element.attrs.push(rule.try_into()?),
                            otherwise => {
                                return Err(anyhow::Error::msg(format!(
                                    "Invalid inner rule while parse open_tag: {:?}",
                                    otherwise
                                )))
                            }
                        }
                    }
                }
                Rule::inner => element.inners.push(rule.try_into()?),
                otherwise => {
                    return Err(anyhow::Error::msg(format!(
                        "Invalid inner rule while parse element: {:?}",
                        otherwise
                    )))
                }
            }
        }

        Ok(element)
    }
}

impl TryFrom<Pair<'_, Rule>> for super::Attr {
    type Error = anyhow::Error;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let mut attr = super::Attr::default();
        for rule in value.into_inner() {
            match rule.as_rule() {
                Rule::attr_name => attr.name = rule.as_str().to_string(),
                Rule::attr_value => attr.value = rule.as_str().to_string(),
                otherwise => {
                    return Err(anyhow::Error::msg(format!(
                        "Invalid inner rule while parse attr: {:?}",
                        otherwise
                    )))
                }
            }
        }

        Ok(attr)
    }
}

impl TryFrom<Pair<'_, Rule>> for super::Node {
    type Error = anyhow::Error;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if let Some(rule) = value.into_inner().next() {
            match rule.as_rule() {
                Rule::element => Ok(Self::Element(rule.try_into()?)),
                Rule::text => Ok(Self::Text(rule.try_into()?)),
                otherwise => Err(anyhow::Error::msg(format!(
                    "Invalid inner rule while parse node: {:?}",
                    otherwise
                ))),
            }
        } else {
            Err(anyhow::Error::msg("Innder rule mustn't be empty"))
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for super::Text {
    type Error = anyhow::Error;

    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        if let Rule::text = value.as_rule() {
            Ok(Self(value.as_str().to_string()))
        } else {
            Err(anyhow::Error::msg(format!(
                "Text rule isn't text: {:?}",
                value.as_rule()
            )))
        }
    }
}
