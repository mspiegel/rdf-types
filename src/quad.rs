use alloc::borrow::ToOwned;
use core::{cmp::Ordering, fmt};

use iref::{Iri, IriBuf};

use thiserror_nostd_notrait::Error;

use crate::{
	interpretation::Interpret,
	vocabulary::{
		ByRef, EmbedIntoVocabulary, EmbeddedIntoVocabulary, ExtractFromVocabulary,
		ExtractedFromVocabulary, TryExtractFromVocabulary,
	},
	GraphLabel, Id, Interpretation, LexicalGraphLabelRef, LexicalObjectRef, LexicalSubjectRef,
	Object, RdfDisplay, Term, Triple,
};

#[cfg(feature = "contextual")]
use contextual::{DisplayWithContext, WithContext};

#[cfg(feature = "contextual")]
use crate::RdfDisplayWithContext;

/// Lexical RDF quad.
pub type LexicalQuad = Quad<Id, IriBuf, Object, GraphLabel>;

/// Lexical RDF quad reference.
pub type LexicalQuadRef<'a> =
	Quad<LexicalSubjectRef<'a>, &'a Iri, LexicalObjectRef<'a>, LexicalGraphLabelRef<'a>>;

/// RDF quad.
#[derive(Clone, Copy, Eq, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quad<S = Term, P = S, O = S, G = S>(pub S, pub P, pub O, pub Option<G>);

impl<S, P, O, G> Quad<S, P, O, G> {
	#[deprecated(since = "0.18.4", note = "please use `as_ref` instead")]
	pub fn borrow_components(&self) -> Quad<&S, &P, &O, &G> {
		self.as_ref()
	}

	/// Borrows each component of the quad.
	pub fn as_ref(&self) -> Quad<&S, &P, &O, &G> {
		Quad(&self.0, &self.1, &self.2, self.3.as_ref())
	}
}

impl<'s, 'p, 'o, 'g, S, P, O, G> Quad<&'s S, &'p P, &'o O, &'g G> {
	pub fn cloned(&self) -> Quad<S, P, O, G>
	where
		S: Clone,
		P: Clone,
		O: Clone,
		G: Clone,
	{
		Quad(
			self.0.clone(),
			self.1.clone(),
			self.2.clone(),
			self.3.cloned(),
		)
	}

	pub fn into_cloned(self) -> Quad<S, P, O, G>
	where
		S: Clone,
		P: Clone,
		O: Clone,
		G: Clone,
	{
		Quad(
			self.0.clone(),
			self.1.clone(),
			self.2.clone(),
			self.3.cloned(),
		)
	}
}

impl<'s, 'p, 'o, 'g, S, P, O, G> Quad<&'s S, &'p P, &'o O, &'g G> {
	pub fn copied(&self) -> Quad<S, P, O, G>
	where
		S: Copy,
		P: Copy,
		O: Copy,
		G: Copy,
	{
		Quad(*self.0, *self.1, *self.2, self.3.copied())
	}

	pub fn into_copied(self) -> Quad<S, P, O, G>
	where
		S: Copy,
		P: Copy,
		O: Copy,
		G: Copy,
	{
		Quad(*self.0, *self.1, *self.2, self.3.copied())
	}
}

impl LexicalQuad {
	pub fn as_lexical_quad_ref(&self) -> LexicalQuadRef {
		Quad(
			self.0.as_lexical_subject_ref(),
			self.1.as_iri(),
			self.2.as_lexical_object_ref(),
			self.3.as_ref().map(GraphLabel::as_graph_label_ref),
		)
	}
}

impl<'a> LexicalQuadRef<'a> {
	pub fn into_owned(self) -> LexicalQuad {
		Quad(
			self.0.into_owned(),
			self.1.to_owned(),
			self.2.into_owned(),
			self.3.map(LexicalGraphLabelRef::into_owned),
		)
	}
}

impl<
		V,
		S: EmbedIntoVocabulary<V>,
		P: EmbedIntoVocabulary<V>,
		O: EmbedIntoVocabulary<V>,
		G: EmbedIntoVocabulary<V>,
	> EmbedIntoVocabulary<V> for Quad<S, P, O, G>
{
	type Embedded = Quad<S::Embedded, P::Embedded, O::Embedded, G::Embedded>;

	fn embed_into_vocabulary(self, vocabulary: &mut V) -> Self::Embedded {
		Quad(
			self.0.embed_into_vocabulary(vocabulary),
			self.1.embed_into_vocabulary(vocabulary),
			self.2.embed_into_vocabulary(vocabulary),
			self.3.embed_into_vocabulary(vocabulary),
		)
	}
}

impl<
		V,
		S: EmbeddedIntoVocabulary<V>,
		P: EmbeddedIntoVocabulary<V>,
		O: EmbeddedIntoVocabulary<V>,
		G: EmbeddedIntoVocabulary<V>,
	> EmbeddedIntoVocabulary<V> for Quad<S, P, O, G>
{
	type Embedded = Quad<S::Embedded, P::Embedded, O::Embedded, G::Embedded>;

	fn embedded_into_vocabulary(&self, vocabulary: &mut V) -> Self::Embedded {
		Quad(
			self.0.embedded_into_vocabulary(vocabulary),
			self.1.embedded_into_vocabulary(vocabulary),
			self.2.embedded_into_vocabulary(vocabulary),
			self.3.embedded_into_vocabulary(vocabulary),
		)
	}
}

impl<S, P, O, G> Quad<S, P, O, G> {
	/// Creates a new quad.
	pub fn new(subject: S, predicate: P, object: O, graph: Option<G>) -> Self {
		Self(subject, predicate, object, graph)
	}

	/// Returns a reference to the subject of the quad,
	/// the first component.
	pub fn subject(&self) -> &S {
		&self.0
	}

	/// Returns a mutable reference to the subject of the quad,
	/// the first component.
	pub fn subject_mut(&mut self) -> &mut S {
		&mut self.0
	}

	/// Turns the quad into its subject,
	/// the first component.
	pub fn into_subject(self) -> S {
		self.0
	}

	/// Returns a reference to the predicate of the quad,
	/// the second component.
	pub fn predicate(&self) -> &P {
		&self.1
	}

	/// Returns a mutable reference to the predicate of the quad,
	/// the second component.
	pub fn predicate_mut(&mut self) -> &mut P {
		&mut self.1
	}

	/// Turns the quad into its predicate,
	/// the second component.
	pub fn into_predicate(self) -> P {
		self.1
	}

	/// Returns a reference to the object of the quad,
	/// the third component.
	pub fn object(&self) -> &O {
		&self.2
	}

	/// Returns a mutable reference to the object of the quad,
	/// the third component.
	pub fn object_mut(&mut self) -> &mut O {
		&mut self.2
	}

	/// Turns the quad into its object,
	/// the third component.
	pub fn into_object(self) -> O {
		self.2
	}

	/// Returns a reference to the graph of the quad,
	/// the fourth component.
	pub fn graph(&self) -> Option<&G> {
		self.3.as_ref()
	}

	/// Returns a mutable reference to the graph of the quad,
	/// the fourth component.
	pub fn graph_mut(&mut self) -> Option<&mut G> {
		self.3.as_mut()
	}

	/// Turns the quad into its graph,
	/// the fourth component.
	pub fn into_graph(self) -> Option<G> {
		self.3
	}

	pub fn into_parts(self) -> (S, P, O, Option<G>) {
		(self.0, self.1, self.2, self.3)
	}

	/// Turns this quad into a triple and its graph component.
	pub fn into_triple(self) -> (Triple<S, P, O>, Option<G>) {
		(Triple(self.0, self.1, self.2), self.3)
	}

	/// Maps the subject with the given function.
	pub fn map_subject<U>(self, f: impl FnOnce(S) -> U) -> Quad<U, P, O, G> {
		Quad(f(self.0), self.1, self.2, self.3)
	}

	/// Maps the subject with the given function.
	pub fn map_predicate<U>(self, f: impl FnOnce(P) -> U) -> Quad<S, U, O, G> {
		Quad(self.0, f(self.1), self.2, self.3)
	}

	/// Maps the subject with the given function.
	pub fn map_object<U>(self, f: impl FnOnce(O) -> U) -> Quad<S, P, U, G> {
		Quad(self.0, self.1, f(self.2), self.3)
	}

	/// Maps the graph with the given function.
	pub fn map_graph<U>(self, f: impl FnOnce(Option<G>) -> Option<U>) -> Quad<S, P, O, U> {
		Quad(self.0, self.1, self.2, f(self.3))
	}

	/// Maps every quad component with the given functions, one for each
	/// component.
	pub fn map_all<S2, P2, O2, G2>(
		self,
		s: impl FnOnce(S) -> S2,
		p: impl FnOnce(P) -> P2,
		o: impl FnOnce(O) -> O2,
		g: impl FnOnce(Option<G>) -> Option<G2>,
	) -> Quad<S2, P2, O2, G2> {
		Quad(s(self.0), p(self.1), o(self.2), g(self.3))
	}
}

impl<T> Quad<T, T, T, T> {
	/// Maps the components with the given function.
	pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Quad<U, U, U, U> {
		Quad(f(self.0), f(self.1), f(self.2), self.3.map(f))
	}
}

impl<S: Interpret<I>, P: Interpret<I>, O: Interpret<I>, G: Interpret<I>, I: Interpretation>
	Interpret<I> for Quad<S, P, O, G>
{
	type Interpreted = Quad<S::Interpreted, P::Interpreted, O::Interpreted, G::Interpreted>;

	fn interpret(self, interpretation: &mut I) -> Self::Interpreted {
		Quad(
			self.0.interpret(interpretation),
			self.1.interpret(interpretation),
			self.2.interpret(interpretation),
			self.3.interpret(interpretation),
		)
	}
}

impl<
		V,
		S: ExtractFromVocabulary<V>,
		P: ExtractFromVocabulary<V>,
		O: ExtractFromVocabulary<V>,
		G: ExtractFromVocabulary<V>,
	> ExtractFromVocabulary<V> for Quad<S, P, O, G>
{
	type Extracted = Quad<S::Extracted, P::Extracted, O::Extracted, G::Extracted>;

	fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
		Quad(
			self.0.extract_from_vocabulary(vocabulary),
			self.1.extract_from_vocabulary(vocabulary),
			self.2.extract_from_vocabulary(vocabulary),
			self.3.extract_from_vocabulary(vocabulary),
		)
	}
}

impl<V, S, P, O, G> ExtractFromVocabulary<V> for ByRef<Quad<S, P, O, G>>
where
	ByRef<S>: ExtractFromVocabulary<V>,
	ByRef<P>: ExtractFromVocabulary<V>,
	ByRef<O>: ExtractFromVocabulary<V>,
	ByRef<G>: ExtractFromVocabulary<V>,
{
	type Extracted = Quad<
		<ByRef<S> as ExtractFromVocabulary<V>>::Extracted,
		<ByRef<P> as ExtractFromVocabulary<V>>::Extracted,
		<ByRef<O> as ExtractFromVocabulary<V>>::Extracted,
		<ByRef<G> as ExtractFromVocabulary<V>>::Extracted,
	>;

	fn extract_from_vocabulary(self, vocabulary: &V) -> Self::Extracted {
		Quad(
			ByRef(self.0 .0).extract_from_vocabulary(vocabulary),
			ByRef(self.0 .1).extract_from_vocabulary(vocabulary),
			ByRef(self.0 .2).extract_from_vocabulary(vocabulary),
			ByRef(self.0 .3).extract_from_vocabulary(vocabulary),
		)
	}
}

impl<
		V,
		S: ExtractedFromVocabulary<V>,
		P: ExtractedFromVocabulary<V>,
		O: ExtractedFromVocabulary<V>,
		G: ExtractedFromVocabulary<V>,
	> ExtractedFromVocabulary<V> for Quad<S, P, O, G>
{
	type Extracted = Quad<S::Extracted, P::Extracted, O::Extracted, G::Extracted>;

	fn extracted_from_vocabulary(&self, vocabulary: &V) -> Self::Extracted {
		Quad(
			self.0.extracted_from_vocabulary(vocabulary),
			self.1.extracted_from_vocabulary(vocabulary),
			self.2.extracted_from_vocabulary(vocabulary),
			self.3.extracted_from_vocabulary(vocabulary),
		)
	}
}

/// Type that can turn a `Quad<S, P, O, G>` into a `Quad`.
pub trait TryExportQuad<S, P, O, G> {
	type Error;

	fn try_export_quad(&self, quad: Quad<S, P, O, G>) -> Result<LexicalQuad, Self::Error>;
}

/// Error returned when calling [`try_extract_from_vocabulary`][1] on a
/// [`Quad`].
///
/// [1]: TryExtractFromVocabulary::try_extract_from_vocabulary
#[derive(Debug, Error)]
pub enum QuadExportFailed<S, P, O, G> {
	#[error("invalid subject: {0}")]
	Subject(S),

	#[error("invalid predicate: {0}")]
	Predicate(P),

	#[error("invalid object: {0}")]
	Object(O),

	#[error("invalid graph label: {0}")]
	Graph(G),
}

impl<
		V,
		S: TryExtractFromVocabulary<V>,
		P: TryExtractFromVocabulary<V>,
		O: TryExtractFromVocabulary<V>,
		G: TryExtractFromVocabulary<V>,
	> TryExtractFromVocabulary<V> for Quad<S, P, O, G>
{
	type Extracted = Quad<S::Extracted, P::Extracted, O::Extracted, G::Extracted>;
	type Error = QuadExportFailed<S::Error, P::Error, O::Error, G::Error>;

	fn try_extract_from_vocabulary(self, vocabulary: &V) -> Result<Self::Extracted, Self::Error> {
		Ok(Quad(
			self.0
				.try_extract_from_vocabulary(vocabulary)
				.map_err(QuadExportFailed::Subject)?,
			self.1
				.try_extract_from_vocabulary(vocabulary)
				.map_err(QuadExportFailed::Predicate)?,
			self.2
				.try_extract_from_vocabulary(vocabulary)
				.map_err(QuadExportFailed::Object)?,
			self.3
				.try_extract_from_vocabulary(vocabulary)
				.map_err(QuadExportFailed::Graph)?,
		))
	}
}

impl<
		S1: PartialEq<S2>,
		P1: PartialEq<P2>,
		O1: PartialEq<O2>,
		G1: PartialEq<G2>,
		S2,
		P2,
		O2,
		G2,
	> PartialEq<Quad<S2, P2, O2, G2>> for Quad<S1, P1, O1, G1>
{
	fn eq(&self, other: &Quad<S2, P2, O2, G2>) -> bool {
		self.0 == other.0
			&& self.1 == other.1
			&& self.2 == other.2
			&& match (&self.3, &other.3) {
				(Some(a), Some(b)) => a == b,
				(None, None) => true,
				_ => false,
			}
	}
}

impl<
		S1: PartialOrd<S2>,
		P1: PartialOrd<P2>,
		O1: PartialOrd<O2>,
		G1: PartialOrd<G2>,
		S2,
		P2,
		O2,
		G2,
	> PartialOrd<Quad<S2, P2, O2, G2>> for Quad<S1, P1, O1, G1>
{
	fn partial_cmp(&self, other: &Quad<S2, P2, O2, G2>) -> Option<Ordering> {
		match self.0.partial_cmp(&other.0) {
			Some(Ordering::Equal) => match self.1.partial_cmp(&other.1) {
				Some(Ordering::Equal) => match self.2.partial_cmp(&other.2) {
					Some(Ordering::Equal) => match (&self.3, &other.3) {
						(Some(a), Some(b)) => a.partial_cmp(b),
						(Some(_), None) => Some(Ordering::Greater),
						(None, Some(_)) => Some(Ordering::Less),
						(None, None) => Some(Ordering::Equal),
					},
					cmp => cmp,
				},
				cmp => cmp,
			},
			cmp => cmp,
		}
	}
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay, G: RdfDisplay> fmt::Display for Quad<S, P, O, G> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(
				f,
				"{} {} {} {}",
				self.0.rdf_display(),
				self.1.rdf_display(),
				self.2.rdf_display(),
				graph.rdf_display()
			),
			None => write!(
				f,
				"{} {} {}",
				self.0.rdf_display(),
				self.1.rdf_display(),
				self.2.rdf_display()
			),
		}
	}
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay, G: RdfDisplay> RdfDisplay for Quad<S, P, O, G> {
	fn rdf_fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(
				f,
				"{} {} {} {}",
				self.0.rdf_display(),
				self.1.rdf_display(),
				self.2.rdf_display(),
				graph.rdf_display()
			),
			None => write!(
				f,
				"{} {} {}",
				self.0.rdf_display(),
				self.1.rdf_display(),
				self.2.rdf_display()
			),
		}
	}
}

#[cfg(feature = "contextual")]
impl<
		S: RdfDisplayWithContext<V>,
		P: RdfDisplayWithContext<V>,
		O: RdfDisplayWithContext<V>,
		G: RdfDisplayWithContext<V>,
		V,
	> DisplayWithContext<V> for Quad<S, P, O, G>
{
	fn fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(
				f,
				"{} {} {} {}",
				self.0.with(vocabulary).rdf_display(),
				self.1.with(vocabulary).rdf_display(),
				self.2.with(vocabulary).rdf_display(),
				graph.with(vocabulary).rdf_display()
			),
			None => write!(
				f,
				"{} {} {}",
				self.0.with(vocabulary).rdf_display(),
				self.1.with(vocabulary).rdf_display(),
				self.2.with(vocabulary).rdf_display()
			),
		}
	}
}

#[cfg(feature = "contextual")]
impl<
		S: RdfDisplayWithContext<V>,
		P: RdfDisplayWithContext<V>,
		O: RdfDisplayWithContext<V>,
		G: RdfDisplayWithContext<V>,
		V,
	> RdfDisplayWithContext<V> for Quad<S, P, O, G>
{
	fn rdf_fmt_with(&self, vocabulary: &V, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(
				f,
				"{} {} {} {}",
				self.0.with(vocabulary).rdf_display(),
				self.1.with(vocabulary).rdf_display(),
				self.2.with(vocabulary).rdf_display(),
				graph.with(vocabulary).rdf_display()
			),
			None => write!(
				f,
				"{} {} {}",
				self.0.with(vocabulary).rdf_display(),
				self.1.with(vocabulary).rdf_display(),
				self.2.with(vocabulary).rdf_display()
			),
		}
	}
}
