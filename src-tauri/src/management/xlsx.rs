//! Text-only OOXML export; rows are streamed to temporary storage, never collected.
use crate::integration::CommandError;
use rust_xlsxwriter::{Workbook, XlsxError};
use std::io::Write;

pub(crate) const MAX_DATA_ROWS: u64 = 1_048_575; // Excel's header consumes one row.

pub(crate) fn validate_total(total: u64) -> Result<(), CommandError> {
    if total > MAX_DATA_ROWS {
        return Err(limit_error(
            "O XLSX comporta até 1.048.575 registros mais o cabeçalho.",
        ));
    }
    Ok(())
}

fn limit_error(context: &str) -> CommandError {
    CommandError {
        code: "export_xlsx_limit".into(),
        message: format!("{context} Restrinja os filtros ou exporte em CSV/JSON, sem esse limite. Nenhum arquivo XLSX foi publicado."),
        retryable: false,
    }
}

fn writer_error(_: XlsxError) -> CommandError {
    CommandError {
        code: "export_io_error".into(),
        message: "Não foi possível gravar o XLSX. Verifique o espaço e as permissões das pastas de destino e temporária.".into(),
        retryable: false,
    }
}

pub(crate) struct XlsxExport {
    workbook: Workbook,
    next_row: u32,
}

impl XlsxExport {
    pub(crate) fn new(headers: &[&str]) -> Result<Self, CommandError> {
        let mut workbook = Workbook::new();
        workbook
            .add_worksheet_with_constant_memory()
            .map_err(writer_error)?;
        let mut export = Self {
            workbook,
            next_row: 0,
        };
        export.write_row(headers)?;
        Ok(export)
    }

    pub(crate) fn write_row(&mut self, cells: &[&str]) -> Result<(), CommandError> {
        if u64::from(self.next_row) > MAX_DATA_ROWS {
            return Err(limit_error(
                "O XLSX excederia 1.048.576 linhas incluindo o cabeçalho.",
            ));
        }
        let sheet = self
            .workbook
            .worksheet_from_index(0)
            .map_err(writer_error)?;
        for (column, cell) in cells.iter().enumerate() {
            sheet
                .write_string(self.next_row, column as u16, *cell)
                .map_err(|err| match err {
                    // The writer enforces Excel's UTF-16 limit, including surrogate pairs.
                    XlsxError::MaxStringLengthExceeded => limit_error(&format!(
                        "A célula da linha {}, coluna {} excede 32.767 unidades UTF-16.",
                        self.next_row + 1,
                        column + 1
                    )),
                    other => writer_error(other),
                })?;
        }
        self.next_row += 1;
        Ok(())
    }

    pub(crate) fn finish(&mut self, writer: &mut (impl Write + Send)) -> Result<(), CommandError> {
        self.workbook.save_to_writer(writer).map_err(writer_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xlsx_rejects_row_and_utf16_limits_without_truncation() {
        assert!(validate_total(MAX_DATA_ROWS).is_ok());
        assert_eq!(
            validate_total(MAX_DATA_ROWS + 1).unwrap_err().code,
            "export_xlsx_limit"
        );
        let mut export = XlsxExport::new(&["Descrição"]).unwrap();
        export.write_row(&[&("😀".repeat(16_383) + "a")]).unwrap();
        let err = export.write_row(&[&"😀".repeat(16_384)]).unwrap_err();
        assert_eq!(err.code, "export_xlsx_limit");
        assert!(err.message.contains("linha 3, coluna 1"));
        assert!(err.message.contains("CSV/JSON"));
    }
}
