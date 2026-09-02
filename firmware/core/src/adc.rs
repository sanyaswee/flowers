//! Embedded HAL does not have the ADC traits, so we have to define and implement our own

/// To be implemented by the ADC in board specific crate
/// PIN => AdcPin type
/// Word => reading data type (e.g. u16, u32)
pub trait AdcProvider<PIN, Word> {
    type Error: defmt::Format;

    /// Perform an asynchronous read on the specified pin
    async fn read(&mut self, pin: &mut PIN) -> Result<Word, Self::Error>;

    /// Get maximum value
    fn max_value(&self) -> Word;
}