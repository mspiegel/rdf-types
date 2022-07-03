//! This is a utility library providing common types
//! when dealing with RDF data:
//! blank node identifier, literal, subject, predicate, object,
//! graph label, gRDF term, triple and quad.
//!
//! The optional feature `loc` provides compatibility
//! with the `locspan` crate to locate every sub-component
//! of a term.
use iref::{Iri, IriBuf};
use std::fmt;

/// Type definitions for RDF types with metadata.
#[cfg(feature = "meta")]
use locspan_derive::*;

mod blankid;
mod literal;
mod term;

#[cfg(feature = "meta")]
pub mod meta;

pub use blankid::*;
pub use literal::*;
pub use term::*;

/// RDF triple.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
	feature = "meta",
	derive(
		StrippedPartialEq,
		StrippedEq,
		StrippedPartialOrd,
		StrippedOrd,
		StrippedHash
	)
)]
pub struct Triple<S = Subject, P = IriBuf, O = Object>(pub S, pub P, pub O);

impl Triple {
	pub fn into_grdf(self) -> GrdfTriple {
		Triple(self.0.into_term(), Term::Iri(self.1), self.2)
	}

	pub fn as_grdf(&self) -> GrdfTripleRef {
		Triple(
			self.0.as_term_ref(),
			TermRef::Iri(self.1.as_iri()),
			self.2.as_term_ref(),
		)
	}
}

impl<S, P, O> Triple<S, P, O> {
	/// Creates a new triple.
	pub fn new(subject: S, predicate: P, object: O) -> Self {
		Self(subject, predicate, object)
	}

	/// Returns a reference to the subject of the triple,
	/// the first component.
	pub fn subject(&self) -> &S {
		&self.0
	}

	/// Turns the triple into its subject,
	/// the first component.
	pub fn into_subject(self) -> S {
		self.0
	}

	/// Returns a reference to the predicate of the triple,
	/// the second component.
	pub fn predicate(&self) -> &P {
		&self.1
	}

	/// Turns the triple into its predicate,
	/// the second component.
	pub fn into_predicate(self) -> P {
		self.1
	}

	/// Returns a reference to the object of the triple,
	/// the third component.
	pub fn object(&self) -> &O {
		&self.2
	}

	/// Turns the triple into its object,
	/// the third component.
	pub fn into_object(self) -> O {
		self.2
	}

	/// Turns the triple into a tuple
	pub fn into_parts(self) -> (S, P, O) {
		(self.0, self.1, self.2)
	}
}

impl<S: fmt::Display, P: fmt::Display, O: fmt::Display> fmt::Display for Triple<S, P, O> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} {} {}", self.0, self.1, self.2)
	}
}

/// RDF triple reference.
pub type TripleRef<'a> = Triple<SubjectRef<'a>, Iri<'a>, ObjectRef<'a>>;

/// gRDF triple.
pub type GrdfTriple = Triple<Term, Term, Term>;

/// gRDF triple reference.
pub type GrdfTripleRef<'a> = Triple<TermRef<'a>, TermRef<'a>, TermRef<'a>>;

/// RDF quad.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
	feature = "meta",
	derive(
		StrippedPartialEq,
		StrippedEq,
		StrippedPartialOrd,
		StrippedOrd,
		StrippedHash
	)
)]
pub struct Quad<S = Subject, P = IriBuf, O = Object, G = GraphLabel>(
	pub S,
	pub P,
	pub O,
	pub Option<G>,
);

impl Quad {
	pub fn into_grdf(self) -> GrdfQuad {
		Quad(
			self.0.into_term(),
			Term::Iri(self.1),
			self.2,
			self.3.map(GraphLabel::into_term),
		)
	}

	pub fn as_grdf(&self) -> GrdfQuadRef {
		Quad(
			self.0.as_term_ref(),
			TermRef::Iri(self.1.as_iri()),
			self.2.as_term_ref(),
			self.3.as_ref().map(GraphLabel::as_term_ref),
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

	/// Turns the quad into its graph,
	/// the fourth component.
	pub fn into_graph(self) -> Option<G> {
		self.3
	}

	pub fn into_parts(self) -> (S, P, O, Option<G>) {
		(self.0, self.1, self.2, self.3)
	}
}

impl<S: fmt::Display, P: fmt::Display, O: fmt::Display, G: fmt::Display> fmt::Display
	for Quad<S, P, O, G>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(f, "{} {} {} {}", self.0, self.1, self.2, graph),
			None => write!(f, "{} {} {}", self.0, self.1, self.2),
		}
	}
}

/// RDF quad reference.
pub type QuadRef<'a> = Quad<SubjectRef<'a>, Iri<'a>, ObjectRef<'a>, GraphLabelRef<'a>>;

/// gRDF quad.
pub type GrdfQuad = Quad<Term, Term, Term, Term>;

/// gRDF quad reference.
pub type GrdfQuadRef<'a> = Quad<TermRef<'a>, TermRef<'a>, TermRef<'a>, TermRef<'a>>;
