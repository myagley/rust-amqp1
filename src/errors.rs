use nom::{IResult, IError};

error_chain!{
    errors {
        Incomplete(n: ::nom::Needed) {
            description("More data required during frame parsing")
            display("More data required during frame parsing: '{:?}'", n)
        }
    }
    foreign_links{
        Io(::std::io::Error);
        NomError(::nom::ErrorKind);
    }
}

pub fn into_result<I, O>(value: IResult<I, O>) -> Result<(I, O)> {
    match value {
        IResult::Done(i, o) => Ok((i, o)),
        IResult::Error(e) => Err(Error::from_kind(ErrorKind::NomError(e))),
        IResult::Incomplete(n) => Err(Error::from_kind(ErrorKind::Incomplete(n)))
    }
}

pub fn into_iresult<I, O>(value: Result<(I, O)>) -> IResult<I, O> {
    match value {
        Ok((i, o)) => IResult::Done(i, o),
        Err(e) => unimplemented!()
    }
}