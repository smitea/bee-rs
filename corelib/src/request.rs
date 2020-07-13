use crate::{Args, Columns, Error, State};
use std::sync::mpsc::SyncSender;

pub struct Request {
    args: Args,
    tx: SyncSender<State>,
}

pub struct Promise<'a> {
    columns: Option<Columns>,
    tx: &'a SyncSender<State>,
}

impl Request {
    pub fn new(args: Args, tx: SyncSender<State>) -> Self {
        Self { args, tx }
    }

    pub fn head(&self, columns: Columns) -> Result<Promise, Error> {
        self.tx.send(State::from(columns))?;
        Ok(Promise {
            tx: &self.tx,
            columns: None,
        })
    }

    pub fn error(&self, err: Error) -> Result<(), Error> {
        self.tx.send(State::from(err))?;

        Ok(())
    }

    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.args
    }
}

impl Drop for Request{
    fn drop(&mut self) {
        let _ = self.tx.send(State::Ok);
        drop(&self.tx);
    }
    
}

impl<'a> Promise<'a> {
    pub fn commit(&mut self, state: State) -> Result<(), Error> {
        if let (State::Process(row), Some(cols)) = (&state, &self.columns) {
            assert_eq!(cols.values.len(), row.values.len());
        } else if let State::Ready(cols) = &state {
            let cols: Columns = cols.clone();
            self.columns = Some(cols);
        }
        self.tx.send(state)?;
        Ok(())
    }
}
