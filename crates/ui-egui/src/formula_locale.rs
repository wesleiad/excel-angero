//! UI formula syntax boundary. Engine commands, control/API callers and XLSX keep English.
use gridcraft_engine::formula::{self, FormulaLocale, ParseError};
use gridcraft_engine::model::Workbook;

pub(crate) fn canonical(
    text: &str,
    locale: FormulaLocale,
    wb: &Workbook,
    sheet: usize,
    cell: gridcraft_engine::core::CellRef,
) -> Result<String, ParseError> {
    // The editor hid the `_xludf.` prefixes of this cell's formula; confirming it unchanged keeps
    // the user-defined calls instead of binding them to the built-ins of the same name.
    if let Some(stored) = wb.sheet(sheet).and_then(|sh| sh.cell(cell)).filter(|c| c.formula.is_some()).map(|c| c.input_text())
        && has_udf_prefix(&stored)
        && text == display(&stored, locale, wb, sheet)
    {
        return Ok(stored);
    }
    if locale == FormulaLocale::En || wb.sheet(sheet).is_some_and(|sh| wb.styles.get(sh.style_id(cell)).num_fmt.as_str() == "@") {
        return Ok(text.to_string());
    }
    // Same formula detection as the engine's `input_to_cell` (no trimming: " =1" is text).
    let body = text;
    if body.starts_with('=')
        || ((body.starts_with('+') || body.starts_with('-')) && body.chars().nth(1).is_some_and(|c| c.is_ascii_alphabetic() || c == '('))
    {
        let known = |name: &str| wb.name(name, sheet).is_some() || wb.table(name).is_some();
        // Match the engine's existing convenience for missing closing parentheses.
        let mut body = body.to_string();
        let mut expr = formula::parse_input(&body, locale, known);
        for _ in 0..8 {
            if expr.is_ok() {
                break;
            }
            body.push(')');
            expr = formula::parse_input(&body, locale, known);
        }
        return Ok(format!("={}", formula::print(&expr?)));
    }
    // Decimal-comma number entry follows the same preference, without rewriting text or dates.
    let trimmed = body.trim();
    // No Brasil o ponto separa milhares: "1.500" é mil e quinhentos.
    let trimmed = if locale == FormulaLocale::Pt && thousands_grouped(trimmed) { trimmed.replace('.', "") } else { trimmed.to_string() };
    let normalized = trimmed.replace(locale.decimal_separator(), ".");
    let number = normalized.strip_suffix('%').unwrap_or(&normalized);
    if number.parse::<f64>().is_ok_and(f64::is_finite) {
        return Ok(normalized);
    }
    Ok(text.to_string())
}

pub(crate) fn display(text: &str, locale: FormulaLocale, wb: &Workbook, sheet: usize) -> String {
    // English (and every language without a formula table) shows the stored text, `_xludf.`
    // prefixes aside, and skips a parse per frame.
    if locale == FormulaLocale::En {
        return hide_udf_prefix(text, locale);
    }
    let Some(body) = text.strip_prefix('=') else { return text.to_string() };
    let shown = formula::parse(body)
        .and_then(|expr| formula::print_input(&expr, locale, |name| wb.name(name, sheet).is_some() || wb.table(name).is_some()))
        .map(|body| format!("={body}"))
        .unwrap_or_else(|_| text.to_string());
    hide_udf_prefix(&shown, locale)
}

/// `1.500`, `-1.234.567,89`, `12.000%`: grupos de três dígitos separados por ponto.
fn thousands_grouped(text: &str) -> bool {
    let body = text.strip_prefix(['-', '+']).unwrap_or(text);
    let body = body.strip_suffix('%').unwrap_or(body);
    let int = body.split_once(',').map_or(body, |(int, _)| int);
    let mut groups = int.split('.');
    let first = groups.next().unwrap_or("");
    let rest: Vec<&str> = groups.collect();
    !rest.is_empty()
        && (1..=3).contains(&first.len())
        && first.bytes().all(|b| b.is_ascii_digit())
        && rest.iter().all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()))
}

const UDF_PREFIX: &str = "_xludf.";

fn has_udf_prefix(text: &str) -> bool {
    text.to_ascii_lowercase().contains(UDF_PREFIX)
}

/// Excel shows a user-defined call such as `_xludf.EOMONTH()` as `=EOMONTH()`. Only function
/// names lose the prefix: text literals and names keep it.
fn hide_udf_prefix(text: &str, locale: FormulaLocale) -> String {
    let Some(body) = text.strip_prefix('=').filter(|b| has_udf_prefix(b)) else { return text.to_string() };
    let Ok(tokens) = formula::lexer::tokenize_locale(body, locale) else { return text.to_string() };
    let mut out = String::from("=");
    let mut from = 0;
    for t in tokens {
        if matches!(t.tok, formula::lexer::Tok::Func(_))
            && body.get(t.start..t.start + UDF_PREFIX.len()).is_some_and(|p| p.eq_ignore_ascii_case(UDF_PREFIX))
        {
            out.push_str(body.get(from..t.start).unwrap_or(""));
            from = t.start + UDF_PREFIX.len();
        }
    }
    out.push_str(body.get(from..).unwrap_or(""));
    out
}

// Ajuda em português das funções mais usadas; as demais mostram o nome em português com a
// descrição original. A pontuação dos parâmetros segue a digitação (`;`).
const PORTUGUESE_HELP: &[(&str, &str, &str)] = &[
    ("SUM", "SOMA(núm1; [núm2]; ...)", "Soma todos os números dos argumentos."),
    ("SUMIF", "SOMASE(intervalo; critérios; [intervalo_soma])", "Soma as células que atendem a um critério."),
    ("SUMIFS", "SOMASES(intervalo_soma; intervalo_critérios1; critérios1; ...)", "Soma as células que atendem a vários critérios."),
    ("SUMPRODUCT", "SOMARPRODUTO(matriz1; [matriz2]; ...)", "Multiplica as matrizes correspondentes e soma os produtos."),
    ("AVERAGE", "MÉDIA(núm1; [núm2]; ...)", "Retorna a média aritmética dos argumentos."),
    ("AVERAGEIF", "MÉDIASE(intervalo; critérios; [intervalo_média])", "Média das células que atendem a um critério."),
    ("AVERAGEIFS", "MÉDIASES(intervalo_média; intervalo_critérios1; critérios1; ...)", "Média das células que atendem a vários critérios."),
    ("COUNT", "CONT.NÚM(valor1; [valor2]; ...)", "Conta quantas células contêm números."),
    ("COUNTA", "CONT.VALORES(valor1; [valor2]; ...)", "Conta quantas células não estão vazias."),
    ("COUNTBLANK", "CONTAR.VAZIO(intervalo)", "Conta quantas células estão vazias."),
    ("COUNTIF", "CONT.SE(intervalo; critérios)", "Conta as células que atendem a um critério."),
    ("COUNTIFS", "CONT.SES(intervalo_critérios1; critérios1; ...)", "Conta as células que atendem a vários critérios."),
    ("MAX", "MÁXIMO(núm1; [núm2]; ...)", "Retorna o maior valor."),
    ("MIN", "MÍNIMO(núm1; [núm2]; ...)", "Retorna o menor valor."),
    ("MAXIFS", "MÁXIMOSES(intervalo_máximo; intervalo_critérios1; critérios1; ...)", "Maior valor entre as células que atendem aos critérios."),
    ("MINIFS", "MÍNIMOSES(intervalo_mínimo; intervalo_critérios1; critérios1; ...)", "Menor valor entre as células que atendem aos critérios."),
    ("MEDIAN", "MED(núm1; [núm2]; ...)", "Retorna a mediana dos números."),
    ("LARGE", "MAIOR(matriz; k)", "Retorna o k-ésimo maior valor."),
    ("SMALL", "MENOR(matriz; k)", "Retorna o k-ésimo menor valor."),
    ("RANK.EQ", "ORDEM.EQ(núm; ref; [ordem])", "Posição de um número em uma lista."),
    ("ROUND", "ARRED(núm; núm_dígitos)", "Arredonda um número para a quantidade de dígitos indicada."),
    ("ROUNDUP", "ARREDONDAR.PARA.CIMA(núm; núm_dígitos)", "Arredonda um número para cima."),
    ("ROUNDDOWN", "ARREDONDAR.PARA.BAIXO(núm; núm_dígitos)", "Arredonda um número para baixo."),
    ("TRUNC", "TRUNCAR(núm; [núm_dígitos])", "Corta as casas decimais sem arredondar."),
    ("INT", "INT(núm)", "Arredonda para baixo até o inteiro mais próximo."),
    ("MOD", "MOD(núm; divisor)", "Retorna o resto da divisão."),
    ("ABS", "ABS(núm)", "Retorna o valor absoluto (sem sinal)."),
    ("PRODUCT", "MULT(núm1; [núm2]; ...)", "Multiplica todos os números."),
    ("POWER", "POTÊNCIA(núm; potência)", "Eleva um número a uma potência."),
    ("SQRT", "RAIZ(núm)", "Retorna a raiz quadrada."),
    ("SUBTOTAL", "SUBTOTAL(núm_função; ref1; ...)", "Subtotal que pode ignorar linhas ocultas."),
    ("IF", "SE(teste_lógico; [valor_se_verdadeiro]; [valor_se_falso])", "Retorna um valor se a condição for verdadeira e outro se for falsa."),
    ("IFS", "SES(teste1; valor1; ...)", "Retorna o valor da primeira condição verdadeira."),
    ("IFERROR", "SEERRO(valor; valor_se_erro)", "Retorna outro valor quando o resultado é um erro."),
    ("IFNA", "SENÃODISP(valor; valor_se_nd)", "Retorna outro valor quando o resultado é #N/D."),
    ("AND", "E(lógico1; [lógico2]; ...)", "VERDADEIRO se todos os argumentos forem verdadeiros."),
    ("OR", "OU(lógico1; [lógico2]; ...)", "VERDADEIRO se algum argumento for verdadeiro."),
    ("NOT", "NÃO(lógico)", "Inverte o valor lógico."),
    ("SWITCH", "PARÂMETRO(expressão; valor1; resultado1; ...; [padrão])", "Compara um valor com uma lista de casos."),
    ("TRUE", "VERDADEIRO()", "Retorna o valor lógico VERDADEIRO."),
    ("FALSE", "FALSO()", "Retorna o valor lógico FALSO."),
    ("VLOOKUP", "PROCV(valor_procurado; matriz_tabela; núm_índice_coluna; [procurar_intervalo])", "Procura um valor na primeira coluna de uma tabela e retorna o valor da mesma linha em outra coluna."),
    ("HLOOKUP", "PROCH(valor_procurado; matriz_tabela; núm_índice_lin; [procurar_intervalo])", "Procura um valor na primeira linha de uma tabela e retorna o valor da mesma coluna em outra linha."),
    ("XLOOKUP", "PROCX(pesquisa_valor; pesquisa_matriz; matriz_retorno; [se_não_encontrado]; [modo_correspondência]; [modo_pesquisa])", "Procura um valor em um intervalo e retorna o item correspondente."),
    ("MATCH", "CORRESP(valor_procurado; matriz_procurada; [tipo_correspondência])", "Retorna a posição de um valor em um intervalo."),
    ("INDEX", "ÍNDICE(matriz; núm_linha; [núm_coluna])", "Retorna o valor de uma posição do intervalo."),
    ("CHOOSE", "ESCOLHER(núm_índice; valor1; [valor2]; ...)", "Escolhe um valor de uma lista pela posição."),
    ("OFFSET", "DESLOC(ref; lins; cols; [altura]; [largura])", "Referência deslocada a partir de um ponto inicial."),
    ("INDIRECT", "INDIRETO(texto_ref; [a1])", "Referência indicada por um texto."),
    ("ROW", "LIN([ref])", "Número da linha de uma referência."),
    ("COLUMN", "COL([ref])", "Número da coluna de uma referência."),
    ("FILTER", "FILTRO(matriz; incluir; [se_vazia])", "Filtra um intervalo pelas condições indicadas."),
    ("SORT", "CLASSIFICAR(matriz; [índice_classificação]; [ordem_classificação]; [por_col])", "Classifica o conteúdo de um intervalo."),
    ("UNIQUE", "ÚNICO(matriz; [por_col]; [exatamente_uma_vez])", "Retorna os valores distintos de um intervalo."),
    ("CONCAT", "CONCAT(texto1; [texto2]; ...)", "Junta vários textos em um só."),
    ("CONCATENATE", "CONCATENAR(texto1; [texto2]; ...)", "Junta vários textos em um só."),
    ("TEXTJOIN", "UNIRTEXTO(delimitador; ignorar_vazio; texto1; ...)", "Junta textos usando um separador."),
    ("LEFT", "ESQUERDA(texto; [núm_caract])", "Retorna os primeiros caracteres de um texto."),
    ("RIGHT", "DIREITA(texto; [núm_caract])", "Retorna os últimos caracteres de um texto."),
    ("MID", "EXT.TEXTO(texto; núm_inicial; núm_caract)", "Retorna caracteres do meio de um texto."),
    ("LEN", "NÚM.CARACT(texto)", "Retorna a quantidade de caracteres de um texto."),
    ("TRIM", "ARRUMAR(texto)", "Remove os espaços extras do texto."),
    ("UPPER", "MAIÚSCULA(texto)", "Converte o texto em maiúsculas."),
    ("LOWER", "MINÚSCULA(texto)", "Converte o texto em minúsculas."),
    ("PROPER", "PRI.MAIÚSCULA(texto)", "Coloca a primeira letra de cada palavra em maiúscula."),
    ("SUBSTITUTE", "SUBSTITUIR(texto; texto_antigo; novo_texto; [núm_da_ocorrência])", "Troca um texto por outro."),
    ("FIND", "PROCURAR(texto_procurado; no_texto; [núm_inicial])", "Posição de um texto dentro de outro (diferencia maiúsculas)."),
    ("SEARCH", "LOCALIZAR(texto_procurado; no_texto; [núm_inicial])", "Posição de um texto dentro de outro."),
    ("TEXT", "TEXTO(valor; formato_texto)", "Converte um valor em texto com o formato indicado."),
    ("VALUE", "VALOR(texto)", "Converte um texto em número."),
    ("TODAY", "HOJE()", "Retorna a data de hoje."),
    ("NOW", "AGORA()", "Retorna a data e a hora atuais."),
    ("DATE", "DATA(ano; mês; dia)", "Monta uma data a partir de ano, mês e dia."),
    ("DAY", "DIA(núm_série)", "Retorna o dia de uma data."),
    ("MONTH", "MÊS(núm_série)", "Retorna o mês de uma data."),
    ("YEAR", "ANO(núm_série)", "Retorna o ano de uma data."),
    ("WEEKDAY", "DIA.DA.SEMANA(núm_série; [retornar_tipo])", "Retorna o dia da semana de uma data."),
    ("EDATE", "DATAM(data_inicial; meses)", "Data que fica alguns meses antes ou depois."),
    ("EOMONTH", "FIMMÊS(data_inicial; meses)", "Último dia do mês, alguns meses antes ou depois."),
    ("NETWORKDAYS", "DIATRABALHOTOTAL(data_inicial; data_final; [feriados])", "Quantidade de dias úteis entre duas datas."),
    ("WORKDAY", "DIATRABALHO(data_inicial; dias; [feriados])", "Data depois de alguns dias úteis."),
    ("DATEDIF", "DATADIF(data_inicial; data_final; unidade)", "Diferença entre duas datas em dias, meses ou anos."),
    ("ISBLANK", "ÉCÉL.VAZIA(valor)", "VERDADEIRO se a célula estiver vazia."),
    ("ISNUMBER", "ÉNÚM(valor)", "VERDADEIRO se o valor for um número."),
    ("ISTEXT", "ÉTEXTO(valor)", "VERDADEIRO se o valor for um texto."),
    ("ISERROR", "ÉERROS(valor)", "VERDADEIRO se o valor for qualquer erro."),
    ("PMT", "PGTO(taxa; nper; vp; [vf]; [tipo])", "Valor da parcela de um empréstimo."),
    ("FV", "VF(taxa; nper; pgto; [vp]; [tipo])", "Valor futuro de um investimento."),
    ("PV", "VP(taxa; nper; pgto; [vf]; [tipo])", "Valor presente de um investimento."),
    ("NPV", "VPL(taxa; valor1; [valor2]; ...)", "Valor presente líquido de fluxos de caixa."),
    ("IRR", "TIR(valores; [estimativa])", "Taxa interna de retorno."),
];

pub(crate) fn signature(name: &str, locale: FormulaLocale) -> Option<String> {
    let canonical = locale.canonical_name(name);
    if locale == FormulaLocale::Pt
        && let Some((_, signature, _)) = PORTUGUESE_HELP.iter().find(|(name, ..)| name.eq_ignore_ascii_case(canonical))
    {
        return Some((*signature).into());
    }
    let signature = crate::formula_bar::function_signature(canonical)?;
    // Sem ajuda traduzida: ao menos o nome aparece em português.
    let shown = match signature.split_once('(') {
        Some((head, rest)) if head.eq_ignore_ascii_case(canonical) => format!("{}({rest}", locale.function_name(canonical)),
        _ => signature,
    };
    Some(shown.replace(',', &locale.list_separator().to_string()))
}

pub(crate) fn description(name: &str, locale: FormulaLocale) -> Option<String> {
    let canonical = locale.canonical_name(name);
    if locale == FormulaLocale::Pt
        && let Some((_, _, description)) = PORTUGUESE_HELP.iter().find(|(name, ..)| name.eq_ignore_ascii_case(canonical))
    {
        return Some((*description).into());
    }
    crate::formula_bar::function_description(canonical)
}

/// Avoid suggesting an alias that would call a workbook/table name instead of the function.
pub(crate) fn completion_name<'a>(name: &'a str, locale: FormulaLocale, wb: &Workbook, sheet: usize) -> &'a str {
    let alias = locale.function_name(name);
    if wb.name(alias, sheet).is_some() || wb.table(alias).is_some() { name } else { alias }
}

/// Engine-generated edit templates use canonical syntax. The only incomplete template is
/// the bare function opening emitted by Insert Function; typed user input never enters here.
pub(crate) fn display_template(text: &str, locale: FormulaLocale, wb: &Workbook, sheet: usize) -> String {
    if let Some(name) = text.strip_prefix('=').and_then(|s| s.strip_suffix('('))
        && !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '.'))
    {
        return format!("={}(", completion_name(name, locale, wb, sheet));
    }
    display(text, locale, wb, sheet)
}
