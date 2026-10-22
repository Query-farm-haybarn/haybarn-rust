use haybarn_loadable_macros::duckdb_entrypoint_c_api_v2;
use haybarn_neo::DuckDBType;
use haybarn_neo::{
    extension::Extension,
    scalar::{ScalarCallbacks, ScalarFunctionBuilder},
    signature::{FunctionParameter, SignatureBuilder},
};

struct AmplifyTest {}

impl ScalarCallbacks for AmplifyTest {
    type BindData = ();
    type InitData = ();

    fn exec(
        &self,
        _bind_data: Option<&Self::BindData>,
        _init_data: Option<&mut Self::InitData>,
        _context: &haybarn_neo::connection::Context,
        vectors: &mut haybarn_neo::data_chunk::VectorCollection,
        output: haybarn_neo::vector::Vector<'_, haybarn_neo::vector::Unknown>,
    ) -> haybarn_neo::Result<()> {
        let input = vectors.get_vector_at::<i32>(0)?.reader()?;

        let mut output = output.cast::<i32>()?;
        output.set_size(input.len())?;
        let mut writer = output.writer()?;

        for (i, item) in input.iter().enumerate() {
            writer.set(i, item.map(|value| value * 10))?;
        }

        Ok(())
    }
}

#[duckdb_entrypoint_c_api_v2(extension_name = "test", api_version = "v2.0.0")]
pub fn entrypoint(extension: &Extension) -> Result<(), Box<dyn ::std::error::Error>> {
    ScalarFunctionBuilder::new(
        "test",
        SignatureBuilder::new()
            .param(FunctionParameter::new::<i32>("input"))
            .return_type(i32::logical_type(extension)?),
        AmplifyTest {},
    )
    .register(extension)?;

    Ok(())
}
