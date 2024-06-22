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
    helper: &'c mut TestHelper,
    thing: Option<T>,
    _phantom: PhantomData<X>,
    _phantom2: PhantomData<Z>,
}

impl<'c, Z, T> Asserter<'c, Z, T, WithoutThing>
where
    Z: PartialEq<T> + Debug,
    T: PartialEq<Z> + Debug,
{
    pub fn new(helper: &'c mut TestHelper) -> Asserter<'c, Z, T, WithoutThing> {
        Asserter {
            helper,
            thing: None,
            _phantom: Default::default(),
            _phantom2: Default::default(),
        }
    }
    pub fn assert_that(self, thing: T) -> Asserter<'c, Z, T, WithThing> {
        Asserter {
            helper: self.helper,
            thing: Some(thing),
            _phantom: Default::default(),
            _phantom2: Default::default(),
        }
    }
}

impl<'c, Z, T> Asserter<'c, Z, T, WithThing>
where
    Z: PartialEq<T> + Debug,
    T: PartialEq<Z> + Debug,
{
    pub async fn is_equals_to(
        &mut self,
        other_thing: Z,
        description: Option<&str>,
    ) -> anyhow::Result<()> {
        // We are WithThing so this is safe
        if other_thing.eq(self.thing.as_ref().unwrap()) {
            Ok(())
        } else {
            let expected = format!("{:#?}", self.thing.as_ref().unwrap());
            let actual = format!("{:#?}", other_thing);

            let diff = similar::TextDiff::from_lines(&expected, &actual);
            let diff = diff.unified_diff().missing_newline_hint(false).to_string();
            self.helper
                .attachment(
                    &format!("Failed: {}", description.unwrap_or("equality comparison.")),
                    Mime::Txt,
                    diff.as_bytes(),
                )
                .await?;
            Err(anyhow!(diff))
        }
    }
}
