use crate::reporter::Mime;
use crate::TestHelper;
use anyhow::anyhow;
use std::fmt::Debug;
use std::marker::PhantomData;

pub struct WithoutThing;

pub struct WithThing;

pub struct Asserter<'c, Z, T, X>
where
    Z: PartialEq<T> + Debug,
    T: PartialEq<Z> + Debug,
{
    pub helper: &'c mut TestHelper,
    pub thing: Option<T>,
    pub _phantom: PhantomData<X>,
    pub _phantom2: PhantomData<Z>,
}
impl<'c, Z, T> Asserter<'c, Z, T, WithoutThing>
where
    Z: PartialEq<T> + Debug,
    T: PartialEq<Z> + Debug,
{
    pub fn assert_that(self, thing: T) -> Asserter<'c, Z, T, WithThing> {
        Asserter {
            helper: self.helper,
            thing: Some(thing),
            _phantom: PhantomData,
            _phantom2: Default::default(),
        }
    }
}

impl<'c, Z, T> Asserter<'c, Z, T, WithThing>
where
    Z: PartialEq<T> + Debug,
    T: PartialEq<Z> + Debug,
{
    pub async fn is_equals_to(&mut self, other_thing: Z) -> anyhow::Result<()> {
        if other_thing.eq(self.thing.as_ref().unwrap()) {
            Ok(())
        } else {
            let expected = format!("{:#?}", self.thing.as_ref().unwrap());
            let actual = format!("{:#?}", other_thing);

            let diff = similar::TextDiff::from_lines(&expected, &actual);
            let diff = diff.unified_diff().missing_newline_hint(false).to_string();
            self.helper
                .attachment("Failed: ADD STEP HERE", Mime::Txt, diff.as_bytes())
                .await?;
            Err(anyhow!(diff))
        }
    }
}
