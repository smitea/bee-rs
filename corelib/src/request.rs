use crate::{Args, Columns, Result, State, Error};
use std::sync::mpsc::SyncSender;
use crate::state::ToData;

pub struct Request {
    args: Args,
    tx: SyncSender<State>,
}

pub struct Promise<'a, T> {
    state: Option<T>,
    inner: Committer<'a>,
}

pub struct Committer<'a> {
    columns: Option<Columns>,
    args: &'a Args,
    tx: &'a SyncSender<State>,
}

impl Committer<'_> {
    pub fn commit(&mut self, state: State) -> Result<()> {
        if let (State::Process(row), Some(cols)) = (&state, &self.columns) {
            assert_eq!(cols.values.len(), row.values.len());
        } else if let State::Ready(cols) = &state {
            let cols: Columns = cols.clone();
            self.columns = Some(cols);
        }
        self.tx.send(state)?;
        Ok(())
    }

    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.args
    }
}

impl Request {
    pub fn new(args: Args, tx: SyncSender<State>) -> Self {
        Self { args, tx }
    }

    pub fn new_commit(&self, columns: Columns) -> Result<Committer> {
        self.tx.send(State::from(columns))?;
        Ok(Committer {
            tx: &self.tx,
            args: &self.args,
            columns: None,
        })
    }

    pub fn head<T: ToData>(&self) -> Result<Promise<T>> {
        let commit = self.new_commit(T::columns())?;
        Ok(Promise { inner: commit, state: None })
    }

    pub fn error(&self, err: Error) -> Result<()> {
        self.tx.send(State::from(err))?;

        Ok(())
    }

    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.args
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        let _ = self.tx.send(State::Ok);
        drop(&self.tx);
    }
}

impl<'a, T> Promise<'a, T> where T: ToData {
    pub fn commit(&mut self, value: T) -> Result<()> {
        self.inner.commit(State::from(value.to_row()))
    }

    pub fn commit_error(&mut self, err: Error) -> Result<()> {
        self.inner.commit(State::from(err))
    }

    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.inner.args
    }
}
