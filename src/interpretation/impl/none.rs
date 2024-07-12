use alloc::borrow::ToOwned;
use iref::{Iri, IriBuf};

use crate::{
	interpretation::{
		BlankIdInterpretation, BlankIdInterpretationMut, IriInterpretation, IriInterpretationMut,
		LiteralInterpretation, LiteralInterpretationMut, ReverseBlankIdInterpretation,
		ReverseIriInterpretation, ReverseLiteralInterpretation,
	},
	BlankId, BlankIdBuf, Id, Interpretation, Literal, Term,
};

impl Interpretation for () {
	type Resource = Term;
}

impl IriInterpretation<IriBuf> for () {
	fn iri_interpretation(&self, iri: &IriBuf) -> Option<Self::Resource> {
		Some(Term::Id(Id::Iri(iri.clone())))
	}
}

impl IriInterpretationMut<IriBuf> for () {
	fn interpret_iri(&mut self, iri: IriBuf) -> Self::Resource {
		Term::Id(Id::Iri(iri))
	}
}

impl IriInterpretation<Iri> for () {
	fn iri_interpretation(&self, iri: &Iri) -> Option<Self::Resource> {
		Some(Term::Id(Id::Iri(iri.to_owned())))
	}
}

impl ReverseIriInterpretation for () {
	type Iri = IriBuf;

	type Iris<'a> = core::option::IntoIter<&'a IriBuf>;

	fn iris_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Iris<'a> {
		match id {
			Term::Id(Id::Iri(iri)) => Some(iri).into_iter(),
			_ => None.into_iter(),
		}
	}
}

impl BlankIdInterpretation<BlankIdBuf> for () {
	fn blank_id_interpretation(&self, blank_id: &BlankIdBuf) -> Option<Self::Resource> {
		Some(Term::Id(Id::Blank(blank_id.to_owned())))
	}
}

impl BlankIdInterpretationMut<BlankIdBuf> for () {
	fn interpret_blank_id(&mut self, blank_id: BlankIdBuf) -> Self::Resource {
		Term::Id(Id::Blank(blank_id))
	}
}

impl BlankIdInterpretation<BlankId> for () {
	fn blank_id_interpretation(&self, blank_id: &BlankId) -> Option<Self::Resource> {
		Some(Term::Id(Id::Blank(blank_id.to_owned())))
	}
}

impl ReverseBlankIdInterpretation for () {
	type BlankId = BlankIdBuf;

	type BlankIds<'a> = core::option::IntoIter<&'a BlankIdBuf>;

	fn blank_ids_of<'a>(&'a self, id: &'a Self::Resource) -> Self::BlankIds<'a> {
		match id {
			Term::Id(Id::Blank(b)) => Some(b).into_iter(),
			_ => None.into_iter(),
		}
	}
}

impl LiteralInterpretation<Literal> for () {
	fn literal_interpretation(&self, literal: &Literal) -> Option<Self::Resource> {
		Some(Term::Literal(literal.clone()))
	}
}

impl LiteralInterpretationMut<Literal> for () {
	fn interpret_literal(&mut self, literal: Literal) -> Self::Resource {
		Term::Literal(literal)
	}
}

impl ReverseLiteralInterpretation for () {
	type Literal = Literal;

	type Literals<'a> = core::option::IntoIter<&'a Literal>;

	fn literals_of<'a>(&'a self, id: &'a Self::Resource) -> Self::Literals<'a> {
		match id {
			Term::Literal(l) => Some(l).into_iter(),
			_ => None.into_iter(),
		}
	}
}
