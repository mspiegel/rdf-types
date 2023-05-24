use crate::{
	InsertIntoVocabulary, InsertedIntoVocabulary, IriVocabulary, LanguageTagVocabulary,
	LiteralVocabularyMut, RdfDisplay,
};
use iref::IriBuf;
use langtag::LanguageTagBuf;
use std::borrow::{Borrow, BorrowMut};
use std::fmt;

#[cfg(feature = "contextual")]
use contextual::DisplayWithContext;

#[cfg(feature = "meta")]
use locspan_derive::{
	StrippedEq, StrippedHash, StrippedOrd, StrippedPartialEq, StrippedPartialOrd,
};

mod map;

pub use map::*;

/// RDF Literal.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "meta",
	derive(
		StrippedPartialEq,
		StrippedEq,
		StrippedPartialOrd,
		StrippedOrd,
		StrippedHash,
	)
)]
pub struct Literal<T = Type<IriBuf, LanguageTagBuf>, S = String> {
	/// Literal value.
	value: S,

	/// Literal type.
	type_: T,
}

impl<T, S> Literal<T, S> {
	pub fn new(value: S, type_: T) -> Self {
		Self { value, type_ }
	}

	pub fn type_(&self) -> &T {
		&self.type_
	}

	pub fn type_mut(&mut self) -> &mut T {
		&mut self.type_
	}

	pub fn into_type_(self) -> T {
		self.type_
	}

	pub fn value(&self) -> &S {
		&self.value
	}

	pub fn value_mut(&mut self) -> &mut S {
		&mut self.value
	}

	pub fn into_value(self) -> S {
		self.value
	}

	pub fn into_parts(self) -> (S, T) {
		(self.value, self.type_)
	}

	pub fn as_str(&self) -> &str
	where
		S: AsRef<str>,
	{
		self.value.as_ref()
	}

	pub fn as_str_mut(&mut self) -> &mut str
	where
		S: AsMut<str>,
	{
		self.value.as_mut()
	}

	pub fn as_bytes(&self) -> &[u8]
	where
		S: AsRef<[u8]>,
	{
		self.value.as_ref()
	}

	pub fn as_bytes_mut(&mut self) -> &mut [u8]
	where
		S: AsMut<[u8]>,
	{
		self.value.as_mut()
	}

	pub fn insert_type_into_vocabulary<V>(self, vocabulary: &mut V) -> Literal<T::Inserted, S>
	where
		T: InsertIntoVocabulary<V>,
	{
		Literal {
			value: self.value,
			type_: self.type_.insert_into_vocabulary(vocabulary),
		}
	}

	pub fn inserted_type_into_vocabulary<V>(&self, vocabulary: &mut V) -> Literal<T::Inserted, S>
	where
		T: InsertedIntoVocabulary<V>,
		S: Clone,
	{
		Literal {
			value: self.value.clone(),
			type_: self.type_.inserted_into_vocabulary(vocabulary),
		}
	}
}

impl<I, L, S> Literal<Type<I, L>, S> {
	pub fn is_lang_string(&self) -> bool {
		self.type_.is_lang_string()
	}

	pub fn lang_tag(&self) -> Option<&L> {
		self.type_.lang_tag()
	}
}

impl<V: LiteralVocabularyMut> InsertIntoVocabulary<V> for Literal<V::Type, V::Value> {
	type Inserted = V::Literal;

	fn insert_into_vocabulary(self, vocabulary: &mut V) -> Self::Inserted {
		vocabulary.insert_literal(&self)
	}
}

impl<V: LiteralVocabularyMut> InsertedIntoVocabulary<V> for Literal<V::Type, V::Value> {
	type Inserted = V::Literal;

	fn inserted_into_vocabulary(&self, vocabulary: &mut V) -> Self::Inserted {
		vocabulary.insert_literal(self)
	}
}

impl<T, S: AsRef<str>> Borrow<str> for Literal<T, S> {
	fn borrow(&self) -> &str {
		self.as_str()
	}
}

impl<T, S: AsRef<str> + AsMut<str>> BorrowMut<str> for Literal<T, S> {
	fn borrow_mut(&mut self) -> &mut str {
		self.as_str_mut()
	}
}

impl<T, S: AsRef<str>> AsRef<str> for Literal<T, S> {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<T, S: AsMut<str>> AsMut<str> for Literal<T, S> {
	fn as_mut(&mut self) -> &mut str {
		self.as_str_mut()
	}
}

impl<T: RdfDisplay + RdfDisplayTypeSeparator, S: RdfDisplay> fmt::Display for Literal<T, S> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.rdf_fmt(f)?;
		self.type_.rdf_fmt_type_separator(f)?;
		self.type_.rdf_fmt(f)
	}
}

impl<T: RdfDisplay + RdfDisplayTypeSeparator, S: RdfDisplay> RdfDisplay for Literal<T, S> {
	fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.rdf_fmt(f)?;
		self.type_.rdf_fmt_type_separator(f)?;
		self.type_.rdf_fmt(f)
	}
}

#[cfg(feature = "contextual")]
impl<T: DisplayWithContext<V> + RdfDisplayTypeSeparator, S: RdfDisplay, V> DisplayWithContext<V>
	for Literal<T, S>
{
	fn fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.rdf_fmt(f)?;
		self.type_.rdf_fmt_type_separator(f)?;
		self.type_.fmt_with(vocabulary, f)
	}
}

#[cfg(feature = "contextual")]
impl<T: crate::RdfDisplayWithContext<V> + RdfDisplayTypeSeparator, S: RdfDisplay, V>
	crate::RdfDisplayWithContext<V> for Literal<T, S>
{
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		self.value.rdf_fmt(f)?;
		self.type_.rdf_fmt_type_separator(f)?;
		self.type_.rdf_fmt_with(vocabulary, f)
	}
}

/// RDF Literal type.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "meta",
	derive(
		StrippedPartialEq,
		StrippedEq,
		StrippedPartialOrd,
		StrippedOrd,
		StrippedHash,
	)
)]
pub enum Type<I = IriBuf, L = LanguageTagBuf> {
	/// Any type.
	Any(I),

	/// Language string.
	LangString(L),
}

impl<I, L> Type<I, L> {
	pub fn is_lang_string(&self) -> bool {
		matches!(self, Self::LangString(_))
	}

	pub fn lang_tag(&self) -> Option<&L> {
		match self {
			Self::LangString(tag) => Some(tag),
			_ => None,
		}
	}
}

impl<V, I: InsertIntoVocabulary<V>, L: InsertIntoVocabulary<V>> InsertIntoVocabulary<V>
	for Type<I, L>
{
	type Inserted = Type<I::Inserted, L::Inserted>;

	fn insert_into_vocabulary(self, vocabulary: &mut V) -> Self::Inserted {
		match self {
			Self::Any(i) => Type::Any(i.insert_into_vocabulary(vocabulary)),
			Self::LangString(l) => Type::LangString(l.insert_into_vocabulary(vocabulary)),
		}
	}
}

pub trait RdfDisplayTypeSeparator {
	fn rdf_fmt_type_separator(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<T, L> RdfDisplayTypeSeparator for Type<T, L> {
	fn rdf_fmt_type_separator(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Any(_) => write!(f, "^^"),
			Self::LangString(_) => write!(f, "@"),
		}
	}
}

impl<T: RdfDisplay, L: RdfDisplay> RdfDisplay for Type<T, L> {
	fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Any(ty) => ty.rdf_fmt(f),
			Self::LangString(tag) => tag.rdf_fmt(f),
		}
	}
}

#[cfg(feature = "contextual")]
impl<T: crate::RdfDisplayWithContext<V>, L: crate::RdfDisplayWithContext<V>, V>
	crate::RdfDisplayWithContext<V> for Type<T, L>
{
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Any(ty) => ty.rdf_fmt_with(vocabulary, f),
			Self::LangString(tag) => tag.rdf_fmt_with(vocabulary, f),
		}
	}
}

/// Type that can be converted into a [`Literal`].
pub trait IntoLiteral {
	/// Literal value type.
	type Value;

	/// Literal type value type.
	type Type;

	/// Turns the value into a `Literal`.
	fn into_literal(self) -> Literal<Self::Type, Self::Value>;
}

impl<T, S> IntoLiteral for Literal<T, S> {
	type Value = S;

	type Type = T;

	fn into_literal(self) -> Self {
		self
	}
}

/// Type that can turn a value into a `Literal`.
pub trait TryExportLiteral<V> {
	type Error;

	fn try_export_literal(self, vocabulary: &V) -> Result<Literal, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
#[error("unknown literal type {0}")]
pub struct UnknownType<I>(pub I);

impl<T: TryExportLiteralType<V>, S: Into<String>, V> TryExportLiteral<V> for Literal<T, S> {
	type Error = T::Error;

	fn try_export_literal(self, vocabulary: &V) -> Result<Literal, Self::Error> {
		Ok(Literal {
			value: self.value.into(),
			type_: self.type_.try_export_literal_type(vocabulary)?,
		})
	}
}

pub trait TryExportLiteralType<V> {
	type Error;

	fn try_export_literal_type(self, vocabulary: &V) -> Result<Type, Self::Error>;
}

impl<V: IriVocabulary + LanguageTagVocabulary> TryExportLiteralType<V>
	for Type<V::Iri, V::LanguageTag>
{
	type Error = ExportError<V::Iri, V::LanguageTag>;

	fn try_export_literal_type(self, vocabulary: &V) -> Result<Type, Self::Error> {
		match self {
			Self::Any(ty) => Ok(Type::Any(
				vocabulary.owned_iri(ty).map_err(ExportError::Iri)?,
			)),
			Self::LangString(tag) => Ok(Type::LangString(
				vocabulary
					.owned_language_tag(tag)
					.map_err(ExportError::LangTag)?,
			)),
		}
	}
}

pub enum ExportError<I, L> {
	Iri(I),
	LangTag(L),
}
