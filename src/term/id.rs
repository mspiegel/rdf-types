use iref::{Iri, IriBuf};
use alloc::borrow::ToOwned;
use core::{cmp::Ordering, fmt, hash::Hash};

#[cfg(feature = "meta")]
use locspan_derive::*;

use crate::{
	vocabulary::{
		BlankIdVocabulary, ByRef, EmbedIntoVocabulary, EmbeddedIntoVocabulary,
		ExtractFromVocabulary, ExtractedFromVocabulary, IriVocabulary,
	},
	BlankId, BlankIdBuf, LexicalGraphLabelRef, LexicalSubjectRef, MaybeBlankId, MaybeIri,
	RdfDisplay, Term, TryAsBlankId, TryAsIri, TryIntoBlankId, TryIntoIri, Vocabulary,
	VocabularyMut,
};

/// RDF node identifier.
///
/// Either a blank node identifier or an IRI.
///
/// # `Hash` implementation
///
/// It is guaranteed that the `Hash` implementation of `Id` is
/// *transparent*, meaning that the hash of `Term::Blank(id)` the same as `id`
/// and the hash of `Subject::Iri(iri)` is the same as `iri`.
#[derive(Clone, Copy, Eq, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "meta",
	derive(StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)
)]
#[cfg_attr(feature = "meta", locspan(stripped(B, I)))]
pub enum Id<I = IriBuf, B = BlankIdBuf> {
	/// Blank node identifier.
	Blank(#[cfg_attr(feature = "meta", locspan(stripped))] B),

	/// IRI.
	Iri(#[cfg_attr(feature = "meta", locspan(stripped))] I),
}

/// Lexical RDF node identifier reference.
pub type LexicalIdRef<'a> = Id<&'a Iri, &'a BlankId>;

impl<I, B> Id<I, B> {
	pub fn is_blank(&self) -> bool {
		matches!(self, Self::Blank(_))
	}

	pub fn is_iri(&self) -> bool {
		matches!(self, Self::Iri(_))
	}

	pub fn as_blank(&self) -> Option<&B> {
		match self {
			Self::Blank(id) => Some(id),
			_ => None,
		}
	}

	pub fn as_iri(&self) -> Option<&I> {
		match self {
			Self::Iri(iri) => Some(iri),
			_ => None,
		}
	}

	pub fn try_into_blank(self) -> Result<B, I> {
		match self {
			Self::Blank(id) => Ok(id),
			Self::Iri(iri) => Err(iri),
		}
	}

	pub fn into_blank(self) -> Option<B> {
		self.try_into_blank().ok()
	}

	pub fn try_into_iri(self) -> Result<I, B> {
		match self {
			Self::Iri(iri) => Ok(iri),
			Self::Blank(b) => Err(b),
		}
	}

	pub fn into_iri(self) -> Option<I> {
		self.try_into_iri().ok()
	}

	/// Converts this id reference into the term `Term::Id(&id)`.
	pub fn as_term<L>(&self) -> Term<&Self, &L> {
		Term::Id(self)
	}

	/// Converts the id into the term `Term::Id(id)`.
	pub fn into_term<L>(self) -> Term<Self, L> {
		Term::Id(self)
	}

	/// Returns a borrowed string representation of the id.
	pub fn as_str(&self) -> &str
	where
		I: AsRef<str>,
		B: AsRef<str>,
	{
		match self {
			Self::Iri(i) => i.as_ref(),
			Self::Blank(i) => i.as_ref(),
		}
	}

	/// Converts an `&Id<I, B>` into an `Id<&I, &B>`.
	pub fn as_ref(&self) -> Id<&I, &B> {
		match self {
			Self::Iri(i) => Id::Iri(i),
			Self::Blank(b) => Id::Blank(b),
		}
	}
}

impl<V, I: EmbedIntoVocabulary<V>, B: EmbedIntoVocabulary<V>> EmbedIntoVocabulary<V> for Id<I, B> {
	type Embedded = Id<I::Embedded, B::Embedded>;

	fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
		match self {
			Self::Iri(i) => Id::Iri(i.embed_into_vocabulary(vocabulary)),
			Self::Blank(b) => Id::Blank(b.embed_into_vocabulary(vocabulary)),
		}
	}
}

impl<V, I: EmbeddedIntoVocabulary<V>, B: EmbeddedIntoVocabulary<V>> EmbeddedIntoVocabulary<V>
	for Id<I, B>
{
	type Embedded = Id<I::Embedded, B::Embedded>;

	fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
		match self {
			Self::Iri(i) => Id::Iri(i.embedded_into_vocabulary(vocabulary)),
			Self::Blank(b) => Id::Blank(b.embedded_into_vocabulary(vocabulary)),
		}
	}
}

impl<V: IriVocabulary + BlankIdVocabulary> ExtractedFromVocabulary<V> for Id<V::Iri, V::BlankId> {
	type Extracted = Id<IriBuf, BlankIdBuf>;

	fn extracted_from_vocabulary(&self, vocabulary: &V) -> Self::Extracted {
		match self {
			Self::Iri(i) => Id::Iri(vocabulary.iri(i).unwrap().to_owned()),
			Self::Blank(b) => Id::Blank(vocabulary.blank_id(b).unwrap().to_owned()),
		}
	}
}

impl<V: IriVocabulary + BlankIdVocabulary> ExtractFromVocabulary<V> for Id<V::Iri, V::BlankId> {
	type Extracted = Id<IriBuf, BlankIdBuf>;

	fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
		match self {
			Self::Iri(i) => Id::Iri(vocabulary.owned_iri(i).ok().unwrap()),
			Self::Blank(b) => Id::Blank(vocabulary.owned_blank_id(b).ok().unwrap()),
		}
	}
}

impl<'a, V: IriVocabulary + BlankIdVocabulary> ExtractFromVocabulary<V>
	for ByRef<Id<&'a V::Iri, &'a V::BlankId>>
{
	type Extracted = Id<IriBuf, BlankIdBuf>;

	fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
		match self.0 {
			Id::Iri(i) => Id::Iri(vocabulary.iri(i).unwrap().to_owned()),
			Id::Blank(b) => Id::Blank(vocabulary.blank_id(b).unwrap().to_owned()),
		}
	}
}

/// Type that can turn an `Id<I, B>` into an `Id`.
pub trait TryExportId<I, B> {
	type Error;

	/// Turns an `Id<I, B>` into an `Id`.
	fn try_export_id(&self, id: Id<I, B>) -> Result<Id, Self::Error>;
}

impl<V: Vocabulary> TryExportId<V::Iri, V::BlankId> for V {
	type Error = Id<V::Iri, V::BlankId>;

	fn try_export_id(&self, id: Id<V::Iri, V::BlankId>) -> Result<Id, Self::Error> {
		match id {
			Id::Iri(i) => self.owned_iri(i).map(Id::Iri).map_err(Id::Iri),
			Id::Blank(b) => self.owned_blank_id(b).map(Id::Blank).map_err(Id::Blank),
		}
	}
}

impl<'a, I, B> Id<&'a I, &'a B> {
	/// Maps an `Id<&I, &B>` into an `Id<I, B>` by cloning the contents of the
	/// id.
	pub fn cloned(self) -> Id<I, B>
	where
		I: Clone,
		B: Clone,
	{
		match self {
			Self::Iri(i) => Id::Iri(i.clone()),
			Self::Blank(b) => Id::Blank(b.clone()),
		}
	}

	/// Maps an `Id<&I, &B>` into an `Id<I, B>` by copying the contents of the
	/// id.
	pub fn copied(self) -> Id<I, B>
	where
		I: Copy,
		B: Copy,
	{
		match self {
			Self::Iri(i) => Id::Iri(*i),
			Self::Blank(b) => Id::Blank(*b),
		}
	}
}

impl Id {
	/// Turns this reference into an `IdRef`.
	#[inline(always)]
	pub fn as_lexical_id_ref(&self) -> LexicalIdRef {
		match self {
			Self::Iri(i) => Id::Iri(i.as_iri()),
			Self::Blank(b) => Id::Blank(b.as_blank_id_ref()),
		}
	}

	/// Alias for `as_id_ref`.
	#[inline(always)]
	pub fn as_lexical_subject_ref(&self) -> LexicalSubjectRef {
		self.as_lexical_id_ref()
	}

	/// Alias for `as_id_ref`.
	#[inline(always)]
	pub fn as_graph_label_ref(&self) -> LexicalGraphLabelRef {
		self.as_lexical_id_ref()
	}

	pub fn inserted_into<V: VocabularyMut>(&self, vocabulary: &mut V) -> Id<V::Iri, V::BlankId> {
		match self {
			Self::Blank(b) => Id::Blank(vocabulary.insert_blank_id(b.as_blank_id_ref())),
			Self::Iri(i) => Id::Iri(vocabulary.insert(i.as_iri())),
		}
	}

	pub fn insert_into<V: VocabularyMut>(self, vocabulary: &mut V) -> Id<V::Iri, V::BlankId> {
		match self {
			Self::Blank(b) => Id::Blank(vocabulary.insert_blank_id(b.as_blank_id_ref())),
			Self::Iri(i) => Id::Iri(vocabulary.insert(i.as_iri())),
		}
	}
}

impl<'a> LexicalIdRef<'a> {
	#[inline(always)]
	pub fn into_owned(self) -> Id {
		match self {
			Self::Iri(i) => Id::Iri(i.to_owned()),
			Self::Blank(b) => Id::Blank(b.to_owned()),
		}
	}
}

impl<I: Hash, B: Hash> Hash for Id<I, B> {
	fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Blank(id) => id.hash(state),
			Self::Iri(i) => i.hash(state),
		}
	}
}

#[cfg(feature = "meta")]
impl<I: Hash, B: Hash> locspan::StrippedHash for Id<I, B> {
	fn stripped_hash<H: core::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Blank(id) => id.hash(state),
			Self::Iri(i) => i.hash(state),
		}
	}
}

impl<I1: PartialEq<I2>, B1: PartialEq<B2>, I2, B2> PartialEq<Id<I2, B2>> for Id<I1, B1> {
	fn eq(&self, other: &Id<I2, B2>) -> bool {
		match (self, other) {
			(Self::Blank(a), Id::Blank(b)) => a == b,
			(Self::Iri(a), Id::Iri(b)) => a == b,
			_ => false,
		}
	}
}

impl<I1: PartialOrd<I2>, B1: PartialOrd<B2>, I2, B2> PartialOrd<Id<I2, B2>> for Id<I1, B1> {
	fn partial_cmp(&self, other: &Id<I2, B2>) -> Option<Ordering> {
		match (self, other) {
			(Self::Blank(a), Id::Blank(b)) => a.partial_cmp(b),
			(Self::Blank(_), _) => Some(Ordering::Less),
			(Self::Iri(a), Id::Iri(b)) => a.partial_cmp(b),
			_ => Some(Ordering::Greater),
		}
	}
}

impl<I: fmt::Display, B: fmt::Display> fmt::Display for Id<I, B> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Blank(id) => id.fmt(f),
			Self::Iri(iri) => write!(f, "{iri}"),
		}
	}
}

#[cfg(feature = "contextual")]
impl<V: crate::vocabulary::IriVocabulary + crate::vocabulary::BlankIdVocabulary>
	contextual::DisplayWithContext<V> for Id<V::Iri, V::BlankId>
{
	fn fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Self::Blank(id) => vocabulary.blank_id(id).unwrap().fmt(f),
			Self::Iri(iri) => write!(f, "{}", vocabulary.iri(iri).unwrap()),
		}
	}
}

impl<I: fmt::Display, B: fmt::Display> RdfDisplay for Id<I, B> {
	fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Blank(id) => id.fmt(f),
			Self::Iri(iri) => write!(f, "<{iri}>"),
		}
	}
}

#[cfg(feature = "contextual")]
impl<V: crate::vocabulary::IriVocabulary + crate::vocabulary::BlankIdVocabulary>
	crate::RdfDisplayWithContext<V> for Id<V::Iri, V::BlankId>
{
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		use fmt::Display;
		match self {
			Self::Blank(id) => vocabulary.blank_id(id).unwrap().fmt(f),
			Self::Iri(iri) => write!(f, "<{}>", vocabulary.iri(iri).unwrap()),
		}
	}
}

#[cfg(feature = "contextual")]
impl<V: crate::vocabulary::IriVocabulary + crate::vocabulary::BlankIdVocabulary>
	contextual::AsRefWithContext<str, V> for Id<V::Iri, V::BlankId>
{
	fn as_ref_with<'a>(&'a self, vocabulary: &'a V) -> &'a str {
		match self {
			Self::Blank(b) => vocabulary.blank_id(b).unwrap().as_str(),
			Self::Iri(i) => vocabulary.iri(i).unwrap().as_str(),
		}
	}
}

/// Types tha may represent a resource identifier.
pub trait MaybeId: MaybeIri + MaybeBlankId {}

impl<T: MaybeIri + MaybeBlankId> MaybeId for T {}

/// Type that can be converted into an `Id`.
pub trait TryAsId: TryAsIri + TryAsBlankId {
	fn try_as_id(&self) -> Option<Id<&Self::Iri, &Self::BlankId>> {
		self.try_as_iri()
			.map(Id::Iri)
			.or_else(|| self.try_as_blank().map(Id::Blank))
	}
}

impl<T: TryAsIri + TryAsBlankId> TryAsId for T {}

/// Type that can be converted into an `Id`.
pub trait TryIntoId: TryIntoIri + TryIntoBlankId {
	fn try_into_id(self) -> Result<Id<Self::Iri, Self::BlankId>, Self> {
		self.try_into_iri()
			.map(Id::Iri)
			.or_else(|t| t.try_into_blank().map(Id::Blank))
	}
}

impl<I, B> TryIntoId for Id<I, B> {
	fn try_into_id(self) -> Result<Self, Self> {
		Ok(self)
	}
}

/// Type that can surely be borrowed as an `Id`.
pub trait AsId: MaybeId {
	fn as_id(&self) -> Id<&Self::Iri, &Self::BlankId>;
}

impl<I, B> AsId for Id<I, B> {
	fn as_id(&self) -> Id<&I, &B> {
		self.as_ref()
	}
}

/// Type that can surely be converted into an `Id`.
pub trait IntoId: MaybeId {
	fn into_id(self) -> Id<Self::Iri, Self::BlankId>;
}

impl<I, B> IntoId for Id<I, B> {
	fn into_id(self) -> Self {
		self
	}
}
