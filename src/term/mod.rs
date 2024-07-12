use crate::interpretation::{Interpret, LiteralInterpretationMut};
use crate::vocabulary::{
	ByRef, EmbedIntoVocabulary, EmbeddedIntoVocabulary, ExtractFromVocabulary,
	ExtractedFromVocabulary, LiteralVocabulary, TryExtractFromVocabulary,
};
use crate::{BlankIdBuf, Literal, RdfDisplay};
use iref::IriBuf;
use core::fmt;
use core::{cmp::Ordering, hash::Hash};

mod id;
mod into;
mod maybe_blank;
mod maybe_iri;
mod maybe_literal;

pub use id::*;
pub use into::*;
pub use maybe_blank::*;
pub use maybe_iri::*;
// pub use maybe_literal::*;

#[cfg(feature = "contextual")]
use contextual::{AsRefWithContext, DisplayWithContext};

#[cfg(feature = "meta")]
use locspan_derive::*;

/// gRDF term.
///
/// Either a node identifier or a literal value.
///
/// # `Hash` implementation
///
/// It is guaranteed that the `Hash` implementation of `Term` is *transparent*,
/// meaning that the hash of `Term::Id(id)` the same as `id` and the hash of
/// `Term::Literal(l)` is the same as `l`.
#[derive(Clone, Copy, Eq, Ord, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
	feature = "meta",
	derive(StrippedPartialEq, StrippedEq, StrippedPartialOrd, StrippedOrd)
)]
pub enum Term<I = Id, L = Literal> {
	/// Node identifier.
	Id(I),

	/// Literal value.
	Literal(L),
}

/// Lexical RDF term reference.
pub type LexicalTermRef<'a> = Term<LexicalIdRef<'a>, &'a Literal>;

impl<I: Hash, L: Hash> Hash for Term<I, L> {
	fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Id(id) => id.hash(state),
			Self::Literal(l) => l.hash(state),
		}
	}
}

#[cfg(feature = "meta")]
impl<I: locspan::StrippedHash, L: locspan::StrippedHash> locspan::StrippedHash for Term<I, L> {
	fn stripped_hash<H: core::hash::Hasher>(&self, state: &mut H) {
		match self {
			Self::Id(id) => id.stripped_hash(state),
			Self::Literal(l) => l.stripped_hash(state),
		}
	}
}

impl<I, L> Term<I, L> {
	pub fn blank(id: I::BlankId) -> Self
	where
		I: FromBlankId,
	{
		Self::Id(I::from_blank(id))
	}

	pub fn iri(iri: I::Iri) -> Self
	where
		I: FromIri,
	{
		Self::Id(I::from_iri(iri))
	}

	pub fn is_id(&self) -> bool {
		matches!(self, Self::Id(_))
	}

	pub fn is_literal(&self) -> bool {
		matches!(self, Self::Literal(_))
	}

	pub fn as_id(&self) -> Option<&I> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}

	pub fn into_id(self) -> Option<I> {
		match self {
			Self::Id(id) => Some(id),
			_ => None,
		}
	}

	pub fn try_into_id(self) -> Result<I, L> {
		match self {
			Self::Id(id) => Ok(id),
			Self::Literal(l) => Err(l),
		}
	}

	pub fn as_literal(&self) -> Option<&L> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}

	pub fn into_literal(self) -> Option<L> {
		match self {
			Self::Literal(lit) => Some(lit),
			_ => None,
		}
	}

	pub fn try_into_literal(self) -> Result<L, I> {
		match self {
			Self::Literal(lit) => Ok(lit),
			Self::Id(id) => Err(id),
		}
	}

	pub fn is_blank(&self) -> bool
	where
		I: TryAsBlankId,
	{
		match self {
			Self::Id(id) => id.is_blank(),
			Self::Literal(_) => false,
		}
	}

	pub fn is_iri(&self) -> bool
	where
		I: TryAsIri,
	{
		match self {
			Self::Id(id) => id.is_iri(),
			Self::Literal(_) => false,
		}
	}

	pub fn as_blank(&self) -> Option<&I::BlankId>
	where
		I: TryAsBlankId,
	{
		match self {
			Self::Id(id) => id.try_as_blank(),
			_ => None,
		}
	}

	pub fn try_into_blank(self) -> Result<I::BlankId, Self>
	where
		I: TryIntoBlankId,
	{
		match self {
			Self::Id(id) => id.try_into_blank().map_err(Self::Id),
			other => Err(other),
		}
	}

	pub fn into_blank(self) -> Option<I::BlankId>
	where
		I: TryIntoBlankId,
	{
		self.try_into_blank().ok()
	}

	pub fn as_iri(&self) -> Option<&I::Iri>
	where
		I: TryAsIri,
	{
		match self {
			Self::Id(id) => id.try_as_iri(),
			_ => None,
		}
	}

	pub fn try_into_iri(self) -> Result<I::Iri, Self>
	where
		I: TryIntoIri,
	{
		match self {
			Self::Id(id) => id.try_into_iri().map_err(Self::Id),
			other => Err(other),
		}
	}

	pub fn into_iri(self) -> Option<I::Iri>
	where
		I: TryIntoIri,
	{
		self.try_into_iri().ok()
	}

	/// Converts from `&Term<I, L>` to `Term<&I, &L>`.
	pub fn as_ref(&self) -> Term<&I, &L> {
		match self {
			Self::Id(id) => Term::Id(id),
			Self::Literal(l) => Term::Literal(l),
		}
	}
}

impl<I: LiteralInterpretationMut<L>, T: Interpret<I, Interpreted = I::Resource>, L> Interpret<I>
	for Term<T, L>
{
	type Interpreted = I::Resource;

	fn interpret(self, interpretation: &mut I) -> Self::Interpreted {
		match self {
			Self::Id(id) => id.interpret(interpretation),
			Self::Literal(l) => interpretation.interpret_literal(l),
		}
	}
}

impl<V: LiteralVocabulary, I: ExtractedFromVocabulary<V>> ExtractedFromVocabulary<V>
	for Term<I, V::Literal>
{
	type Extracted = Term<I::Extracted, Literal>;

	fn extracted_from_vocabulary(&self, vocabulary: &V) -> Self::Extracted {
		match self {
			Self::Id(i) => Term::Id(i.extracted_from_vocabulary(vocabulary)),
			Self::Literal(l) => Term::Literal(
				vocabulary
					.literal(l)
					.unwrap()
					.extract_from_vocabulary(vocabulary),
			),
		}
	}
}

impl<V: LiteralVocabulary, I: ExtractFromVocabulary<V>> ExtractFromVocabulary<V>
	for Term<I, V::Literal>
{
	type Extracted = Term<I::Extracted, Literal>;

	fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
		match self {
			Self::Id(i) => Term::Id(i.extract_from_vocabulary(vocabulary)),
			Self::Literal(l) => Term::Literal(
				vocabulary
					.owned_literal(l)
					.ok()
					.unwrap()
					.extract_from_vocabulary(vocabulary),
			),
		}
	}
}

impl<'a, V: LiteralVocabulary, I> ExtractFromVocabulary<V> for ByRef<Term<I, &'a V::Literal>>
where
	ByRef<I>: ExtractFromVocabulary<V>,
{
	type Extracted = Term<<ByRef<I> as ExtractFromVocabulary<V>>::Extracted, Literal>;

	fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
		match self.0 {
			Term::Id(i) => Term::Id(ByRef(i).extract_from_vocabulary(vocabulary)),
			Term::Literal(l) => Term::Literal(
				vocabulary
					.literal(l)
					.unwrap()
					.extract_from_vocabulary(vocabulary),
			),
		}
	}
}

impl<V, I: TryExtractFromVocabulary<V>, L: TryExtractFromVocabulary<V>> TryExtractFromVocabulary<V>
	for Term<I, L>
{
	type Extracted = Term<I::Extracted, L::Extracted>;

	type Error = Term<I::Error, L::Error>;

	fn try_extract_from_vocabulary(self, vocabulary: &V) -> Result<Self::Extracted, Self::Error> {
		match self {
			Self::Id(i) => i
				.try_extract_from_vocabulary(vocabulary)
				.map_err(Term::Id)
				.map(Term::Id),
			Self::Literal(l) => l
				.try_extract_from_vocabulary(vocabulary)
				.map_err(Term::Literal)
				.map(Term::Literal),
		}
	}
}

impl<'a, I, L> Term<&'a I, &'a L> {
	pub fn cloned(self) -> Term<I, L>
	where
		I: Clone,
		L: Clone,
	{
		match self {
			Self::Id(id) => Term::Id(id.clone()),
			Self::Literal(l) => Term::Literal(l.clone()),
		}
	}

	pub fn copied(self) -> Term<I, L>
	where
		I: Copy,
		L: Copy,
	{
		match self {
			Self::Id(id) => Term::Id(*id),
			Self::Literal(l) => Term::Literal(*l),
		}
	}
}

impl Term {
	#[inline(always)]
	pub fn as_lexical_term_ref(&self) -> LexicalTermRef {
		match self {
			Self::Id(id) => Term::Id(id.as_lexical_id_ref()),
			Self::Literal(l) => Term::Literal(l),
		}
	}

	/// Alias for [`Self::as_lexical_term_ref`].
	#[inline(always)]
	pub fn as_lexical_object_ref(&self) -> LexicalObjectRef {
		self.as_lexical_term_ref()
	}
}

impl<V, I: EmbedIntoVocabulary<V>, L: EmbedIntoVocabulary<V>> EmbedIntoVocabulary<V>
	for Term<I, L>
{
	type Embedded = Term<I::Embedded, L::Embedded>;

	fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
		match self {
			Self::Id(id) => Term::Id(id.embed_into_vocabulary(vocabulary)),
			Self::Literal(l) => Term::Literal(l.embed_into_vocabulary(vocabulary)),
		}
	}
}

impl<V, I: EmbeddedIntoVocabulary<V>, L: EmbeddedIntoVocabulary<V>> EmbeddedIntoVocabulary<V>
	for Term<I, L>
{
	type Embedded = Term<I::Embedded, L::Embedded>;

	fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
		match self {
			Self::Id(id) => Term::Id(id.embedded_into_vocabulary(vocabulary)),
			Self::Literal(l) => Term::Literal(l.embedded_into_vocabulary(vocabulary)),
		}
	}
}

impl<I1: PartialEq<I2>, L1: PartialEq<L2>, I2, L2> PartialEq<Term<I2, L2>> for Term<I1, L1> {
	fn eq(&self, other: &Term<I2, L2>) -> bool {
		match (self, other) {
			(Self::Id(a), Term::Id(b)) => a == b,
			(Self::Literal(a), Term::Literal(b)) => a == b,
			_ => false,
		}
	}
}

impl<I1: PartialOrd<I2>, L1: PartialOrd<L2>, I2, L2> PartialOrd<Term<I2, L2>> for Term<I1, L1> {
	fn partial_cmp(&self, other: &Term<I2, L2>) -> Option<Ordering> {
		match (self, other) {
			(Self::Id(a), Term::Id(b)) => a.partial_cmp(b),
			(Self::Id(_), Term::Literal(_)) => Some(Ordering::Less),
			(Self::Literal(a), Term::Literal(b)) => a.partial_cmp(b),
			(Self::Literal(_), Term::Id(_)) => Some(Ordering::Greater),
		}
	}
}

impl<I: fmt::Display, L: fmt::Display> fmt::Display for Term<I, L> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(f),
		}
	}
}

impl<I: RdfDisplay, L: RdfDisplay> RdfDisplay for Term<I, L> {
	fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => id.rdf_fmt(f),
			Self::Literal(lit) => lit.rdf_fmt(f),
		}
	}
}

#[cfg(feature = "contextual")]
impl<I: DisplayWithContext<V>, L: DisplayWithContext<V>, V> DisplayWithContext<V> for Term<I, L> {
	fn fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => id.fmt_with(vocabulary, f),
			Self::Literal(lit) => lit.fmt_with(vocabulary, f),
		}
	}
}

#[cfg(feature = "contextual")]
impl<I: crate::RdfDisplayWithContext<V>, L: crate::RdfDisplayWithContext<V>, V>
	crate::RdfDisplayWithContext<V> for Term<I, L>
{
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Id(id) => id.rdf_fmt_with(vocabulary, f),
			Self::Literal(lit) => lit.rdf_fmt_with(vocabulary, f),
		}
	}
}

#[cfg(feature = "contextual")]
impl<I: AsRefWithContext<str, V>, L: AsRef<str>, V> AsRefWithContext<str, V> for Term<I, L> {
	fn as_ref_with<'a>(&'a self, vocabulary: &'a V) -> &'a str {
		match self {
			Self::Id(id) => id.as_ref_with(vocabulary),
			Self::Literal(l) => l.as_ref(),
		}
	}
}

impl<'a> LexicalTermRef<'a> {
	pub fn into_owned(self) -> Term {
		match self {
			Self::Id(id) => Term::Id(id.into_owned()),
			Self::Literal(l) => Term::Literal(l.clone()),
		}
	}
}

/// RDF triple/quad subject.
pub type Subject<I = IriBuf, B = BlankIdBuf> = Id<I, B>;

/// Lexical RDF subject reference.
pub type LexicalSubjectRef<'a> = LexicalIdRef<'a>;

/// RDF triple/quad object.
pub type Object<I = Id, L = Literal> = Term<I, L>;

/// Lexical RDF object reference.
pub type LexicalObjectRef<'a> = LexicalTermRef<'a>;

/// RDF quad graph Label.
pub type GraphLabel<I = IriBuf, B = BlankIdBuf> = Id<I, B>;

/// Lexical RDF graph label reference.
pub type LexicalGraphLabelRef<'a> = LexicalIdRef<'a>;

/// Type that can be interpreted as an RDF term.
pub trait AsRdfTerm<I, B, L> {
	/// Returns this value as an RDF term.
	fn as_rdf_term(&self) -> Term<Id<&I, &B>, &L>;
}

impl<I, B, L> AsRdfTerm<I, B, L> for Id<I, B> {
	fn as_rdf_term(&self) -> Term<Id<&I, &B>, &L> {
		Term::Id(self.as_ref())
	}
}

impl<I, B, L> AsRdfTerm<I, B, L> for Term<Id<I, B>, L> {
	fn as_rdf_term(&self) -> Term<Id<&I, &B>, &L> {
		match self {
			Self::Id(id) => Term::Id(id.as_ref()),
			Self::Literal(l) => Term::Literal(l),
		}
	}
}
