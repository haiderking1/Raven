use super::Policy;
use std::time::{Duration, Instant};

type Schedule = super::Schedule<()>;

mod adaptive;
mod budget;
mod callbacks;
mod deadline;
mod lifecycle;
mod redraw;
mod tickets;
