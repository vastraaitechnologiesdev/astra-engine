//! HTML5 tree construction insertion modes.

/// The active insertion mode of the HTML tree builder state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InsertionMode {
    /// Initial insertion mode before any DOCTYPE or root elements.
    #[default]
    Initial,
    /// Mode before the root `<html>` element is inserted.
    BeforeHtml,
    /// Mode before the `<head>` element is opened.
    BeforeHead,
    /// Mode when processing elements inside `<head>`.
    InHead,
    /// Mode after `<head>` has closed, before `<body>` is encountered.
    AfterHead,
    /// Main document body insertion mode.
    InBody,
    /// Mode for raw text and RCDATA elements (e.g. `<title>`, `<style>`, `<textarea>`).
    Text,
    /// Mode after `</body>` has been closed.
    AfterBody,
    /// Mode after `</html>` has been closed.
    AfterAfterBody,

    // --- Table Insertion Modes ---
    /// Mode when directly inside a `<table>`.
    InTable,
    /// Mode when buffering character tokens inside table context.
    InTableText,
    /// Mode when inside a `<caption>`.
    InCaption,
    /// Mode when inside a `<colgroup>`.
    InColumnGroup,
    /// Mode when inside `<tbody>`, `<thead>`, or `<tfoot>`.
    InTableBody,
    /// Mode when inside a `<tr>`.
    InRow,
    /// Mode when inside a `<td>` or `<th>`.
    InCell,
}
