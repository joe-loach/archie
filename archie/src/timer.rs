use instant::{Duration, Instant};

struct Buffer<T, const N: usize> {
    arr: [T; N],
    size: usize,
    head: usize,
}

impl<T: Default + Copy, const N: usize> Buffer<T, N> {
    fn new() -> Self {
        Buffer {
            arr: [T::default(); N],
            size: 0,
            head: 0,
        }
    }

    fn push(&mut self, item: T) {
        self.head = (self.head + 1) % N;
        self.arr[self.head] = item;
        self.size = (self.size + 1).min(N);
    }

    fn as_slice(&self) -> &[T] {
        &self.arr[..self.size]
    }
}

pub struct InnerTimer {
    start: Instant,
    last: Option<Instant>,
    curr: Option<Instant>,
    buffer: Box<Buffer<Duration, 400>>,
}

pub enum Timer {
    Started(InnerTimer),
    Stopped,
}

impl Timer {
    pub(crate) fn new() -> Self {
        Timer::Stopped
    }

    pub(crate) fn start(&mut self) {
        *self = Timer::Started(InnerTimer {
            start: Instant::now(),
            last: None,
            curr: None,
            buffer: Box::new(Buffer::new()),
        });
    }

    pub(crate) fn tick(&mut self) -> Duration {
        match self {
            Timer::Started(InnerTimer {
                start, last, curr, buffer, ..
            }) => {
                let now = Instant::now();
                let last = core::mem::replace(last, *curr);
                *curr = Some(now);
                let last = last.unwrap_or(*start);
                let dur = now.duration_since(last);
                buffer.push(dur);
                dur
            }
            Timer::Stopped => Duration::ZERO,
        }
    }

    pub fn delta(&self) -> Duration {
        match self {
            Timer::Started(InnerTimer {
                start, last, curr, ..
            }) => {
                let last = last.unwrap_or(*start);
                let curr = curr.unwrap_or_else(Instant::now);
                curr.duration_since(last)
            }
            Timer::Stopped => Duration::ZERO,
        }
    }

    pub fn elapsed(&self) -> Duration {
        match self {
            Timer::Started(InnerTimer { start, .. }) => start.elapsed(),
            Timer::Stopped => Duration::ZERO,
        }
    }

    pub fn average(&self) -> Duration {
        match self {
            Timer::Started(InnerTimer { buffer, .. }) => {
                let durations = buffer.as_slice();
                durations.iter().sum::<Duration>() / (durations.len() as u32 * 2)
            }
            Timer::Stopped => Duration::ZERO,
        }
    }

    pub fn timings(&self) -> &[Duration] {
        if let Timer::Started(InnerTimer { buffer, .. }) = self {
            buffer.as_slice()
        } else {
            &[]
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn timing() {
    let mut t = Timer::new();

    assert!(t.tick().is_zero());
    assert!(t.delta().is_zero());
    assert!(t.elapsed().is_zero());

    t.start();

    assert!(!t.elapsed().is_zero());
    let dt = t.tick();
    assert!(!dt.is_zero());
    assert_eq!(dt, t.delta());
}
