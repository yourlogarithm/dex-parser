use scroll::{ctx, Pread, Sleb128, Uleb128};
use std::ops::Deref;

use getset::Getters;

use crate::{
    code::{CatchHandler, ExceptionType},
    error::Error,
    jtype::TypeId,
    uint, ulong, ushort,
};

pub trait EncodedItem {
    /// Returns the id of the encoded item.
    fn id(&self) -> ulong;
}

#[derive(Getters)]
#[get = "pub"]
pub struct EncodedItemArray<T> {
    inner: Vec<T>,
}

impl<T> Deref for EncodedItemArray<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: EncodedItem> EncodedItemArray<T> {
    pub(crate) fn iter(self) -> impl Iterator<Item = T> {
        self.inner.into_iter()
    }
}

pub(crate) struct EncodedItemArrayCtx<'a, S: AsRef<[u8]>> {
    dex: &'a super::Dex<S>,
    len: usize,
}

impl<'a, S: AsRef<[u8]>> EncodedItemArrayCtx<'a, S> {
    pub(crate) fn new(dex: &'a super::Dex<S>, len: usize) -> Self {
        Self { dex, len }
    }
}

impl<'a, S: AsRef<[u8]>> Copy for EncodedItemArrayCtx<'a, S> {}

impl<'a, S: AsRef<[u8]>> Clone for EncodedItemArrayCtx<'a, S> {
    fn clone(&self) -> Self {
        Self {
            dex: self.dex,
            len: self.len,
        }
    }
}

impl<'a, S, T: 'a> ctx::TryFromCtx<'a, EncodedItemArrayCtx<'a, S>> for EncodedItemArray<T>
where
    S: AsRef<[u8]>,
    T: EncodedItem + ctx::TryFromCtx<'a, ulong, Error = Error>,
{
    type Error = Error;

    fn try_from_ctx(
        source: &'a [u8],
        ctx: EncodedItemArrayCtx<'a, S>,
    ) -> super::Result<(Self, usize)> {
        let len = ctx.len;
        let mut prev = 0;
        let offset = &mut 0;
        let mut inner = Vec::with_capacity(len);
        for _ in 0..len {
            let encoded_item: T = source.gread_with(offset, prev)?;
            prev = encoded_item.id();
            inner.push(encoded_item);
        }
        Ok((EncodedItemArray { inner }, *offset))
    }
}

#[derive(Debug)]
pub(crate) struct EncodedCatchHandlers {
    inner: Vec<(usize, EncodedCatchHandler)>,
}

impl EncodedCatchHandlers {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &(usize, EncodedCatchHandler)> {
        self.inner.iter()
    }

    pub(crate) fn find(&self, handler_offset: ushort) -> Option<&EncodedCatchHandler> {
        self.iter()
            .find(|p| p.0 == handler_offset as usize)
            .map(|p| &p.1)
    }
}

#[derive(Debug)]
pub(crate) struct EncodedCatchHandler {
    handlers: Vec<CatchHandler>,
}

impl EncodedCatchHandler {
    pub(crate) fn handlers(&self) -> Vec<CatchHandler> {
        self.handlers.to_vec()
    }
}

impl<'a, S> ctx::TryFromCtx<'a, &super::Dex<S>> for EncodedCatchHandler
where
    S: AsRef<[u8]>,
{
    type Error = crate::error::Error;

    fn try_from_ctx(source: &'a [u8], dex: &super::Dex<S>) -> super::Result<(Self, usize)> {
        let offset = &mut 0;
        let size = Sleb128::read(source, offset)?;
        let type_addr_pairs: Vec<EncodedTypeAddrPair> =
            try_gread_vec_with!(source, offset, size.abs(), ());
        let mut handlers: Vec<CatchHandler> = type_addr_pairs
            .into_iter()
            .map(|type_addr_pair| {
                Ok(CatchHandler {
                    exception: ExceptionType::Ty(dex.get_type(type_addr_pair.type_id)?),
                    addr: type_addr_pair.addr,
                })
            })
            .collect::<super::Result<_>>()?;
        if size <= 0 {
            let all_handler_addr = Uleb128::read(source, offset)?;
            handlers.push(CatchHandler {
                exception: ExceptionType::BaseException,
                addr: all_handler_addr as ulong,
            });
        }
        Ok((Self { handlers }, *offset))
    }
}

impl<'a, S> ctx::TryFromCtx<'a, &super::Dex<S>> for EncodedCatchHandlers
where
    S: AsRef<[u8]>,
{
    type Error = crate::error::Error;

    fn try_from_ctx(source: &'a [u8], dex: &super::Dex<S>) -> super::Result<(Self, usize)> {
        let offset = &mut 0;
        let encoded_handler_size = Uleb128::read(source, offset)?;
        let mut encoded_catch_handlers = Vec::with_capacity(encoded_handler_size as usize);
        for _ in 0..encoded_handler_size {
            let off = *offset;
            let encoded_catch_handler = source.gread_with(offset, dex)?;
            encoded_catch_handlers.push((off, encoded_catch_handler));
        }
        Ok((
            Self {
                inner: encoded_catch_handlers,
            },
            *offset,
        ))
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct EncodedTypeAddrPair {
    pub(crate) type_id: TypeId,
    pub(crate) addr: ulong,
}

impl<'a> ctx::TryFromCtx<'a, ()> for EncodedTypeAddrPair {
    type Error = crate::error::Error;

    fn try_from_ctx(source: &'a [u8], _: ()) -> super::Result<(Self, usize)> {
        let offset = &mut 0;
        let type_id = Uleb128::read(source, offset)?;
        let addr = Uleb128::read(source, offset)?;
        Ok((
            Self {
                type_id: type_id as uint,
                addr,
            },
            *offset,
        ))
    }
}
