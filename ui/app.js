"use strict";

const invoke = window.__TAURI__.core.invoke;

const $ = (id) => document.getElementById(id);

let DATA = null;
let checkedStrata = new Set();
let checkedSexes = new Set();
let strataInitialized = false;
let sexesInitialized = false;

const SEX_LABEL = { F: "F (women)", M: "M (men)" };

const MIN_EVENTS = 10;

function apparentVE(cell) {
  if (cell.population_exposed == null || cell.population_unexposed == null) return null;
  if (!cell.population_exposed || !cell.population_unexposed || !cell.target_unexposed) return null;
  const rateExposed = cell.target_exposed / cell.population_exposed;
  const rateUnexposed = cell.target_unexposed / cell.population_unexposed;
  return 1 - rateExposed / rateUnexposed;
}

function correctedVE(cell) {
  if (!cell.reference_exposed || !cell.target_unexposed) return null;
  const oddsRatio = (cell.target_exposed * cell.reference_unexposed) / (cell.reference_exposed * cell.target_unexposed);
  return 1 - oddsRatio;
}

function isReliable(cell) {
  return Math.min(cell.target_exposed, cell.reference_exposed, cell.target_unexposed, cell.reference_unexposed) >= MIN_EVENTS;
}

function emptyCell() {
  return { target_exposed: 0, reference_exposed: 0, target_unexposed: 0, reference_unexposed: 0, population_exposed: 0, population_unexposed: 0 };
}

function addCell(a, b) {
  const popExposed = a.population_exposed == null || b.population_exposed == null ? null : a.population_exposed + b.population_exposed;
  const popUnexposed = a.population_unexposed == null || b.population_unexposed == null ? null : a.population_unexposed + b.population_unexposed;
  return {
    target_exposed: a.target_exposed + b.target_exposed,
    reference_exposed: a.reference_exposed + b.reference_exposed,
    target_unexposed: a.target_unexposed + b.target_unexposed,
    reference_unexposed: a.reference_unexposed + b.reference_unexposed,
    population_exposed: popExposed,
    population_unexposed: popUnexposed,
  };
}

function sumRange(stratum, startIdx, endIdx) {
  let cell = emptyCell();
  for (let i = startIdx; i <= endIdx; i++) cell = addCell(cell, stratum.cells[i]);
  return cell;
}

function sumStrata(strata, startIdx, endIdx) {
  let cell = emptyCell();
  for (const s of strata) cell = addCell(cell, sumRange(s, startIdx, endIdx));
  return cell;
}

function showError(title, messages) {
  $("errorTitle").textContent = title;
  const list = $("errorList");
  list.innerHTML = "";
  for (const m of messages) {
    const li = document.createElement("li");
    li.textContent = m;
    list.appendChild(li);
  }
  $("errorPanel").hidden = false;
}

function clearError() {
  $("errorPanel").hidden = true;
}

function handleOutcome(outcome) {
  if (outcome.kind === "Dataset") {
    clearError();
    DATA = outcome.detail;
    strataInitialized = false;
    sexesInitialized = false;
    $("uploadView").hidden = true;
    $("appView").hidden = false;
    initAppView();
    return;
  }
  if (outcome.kind === "MissingColumns") {
    showError("Missing required column(s)", [outcome.detail]);
  } else if (outcome.kind === "RowErrors") {
    showError(`${outcome.detail.length} row(s) failed validation`, outcome.detail);
  } else if (outcome.kind === "Io") {
    showError("Could not read this file", [outcome.detail]);
  }
}

async function loadFromContents(contents) {
  const outcome = await invoke("load_dataset_from_contents", { contents });
  handleOutcome(outcome);
}

function initUploadView() {
  $("chooseFileBtn").addEventListener("click", async () => {
    const outcome = await invoke("pick_and_load_csv");
    if (outcome) handleOutcome(outcome);
  });

  $("reloadBtn").addEventListener("click", () => {
    DATA = null;
    $("appView").hidden = true;
    $("uploadView").hidden = false;
    clearError();
  });

  const dropZone = $("dropZone");
  dropZone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropZone.classList.add("drag-over");
  });
  dropZone.addEventListener("dragleave", () => dropZone.classList.remove("drag-over"));
  dropZone.addEventListener("drop", async (e) => {
    e.preventDefault();
    dropZone.classList.remove("drag-over");
    const file = e.dataTransfer.files && e.dataTransfer.files[0];
    if (!file) return;
    const contents = await file.text();
    await loadFromContents(contents);
  });
}

function fillPeriodSelects() {
  const start = $("startPeriod"), end = $("endPeriod");
  start.innerHTML = "";
  end.innerHTML = "";
  for (const p of DATA.periods) {
    start.add(new Option(p, p));
    end.add(new Option(p, p));
  }
  start.value = DATA.periods[0];
  end.value = DATA.periods[DATA.periods.length - 1];
}

function renderStratumList() {
  const list = $("stratumList");
  list.innerHTML = "";
  if (!strataInitialized) {
    checkedStrata = new Set(DATA.strata.map((s) => s.name));
    strataInitialized = true;
  }
  for (const s of DATA.strata) {
    const wrap = document.createElement("label");
    wrap.className = "check";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = checkedStrata.has(s.name);
    cb.addEventListener("change", () => {
      if (cb.checked) checkedStrata.add(s.name); else checkedStrata.delete(s.name);
      render();
    });
    wrap.appendChild(cb);
    wrap.appendChild(document.createTextNode(" " + s.name));
    list.appendChild(wrap);
  }
}

function renderSexList() {
  const list = $("sexList");
  list.innerHTML = "";
  if (!sexesInitialized) {
    checkedSexes = new Set(DATA.sexes);
    sexesInitialized = true;
  }
  for (const sex of DATA.sexes) {
    const wrap = document.createElement("label");
    wrap.className = "check";
    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = checkedSexes.has(sex);
    cb.addEventListener("change", () => {
      if (cb.checked) checkedSexes.add(sex); else checkedSexes.delete(sex);
      render();
    });
    wrap.appendChild(cb);
    wrap.appendChild(document.createTextNode(" " + (SEX_LABEL[sex] || sex)));
    list.appendChild(wrap);
  }
}

function cellsFor(stratum) {
  let cells = DATA.periods.map(() => emptyCell());
  for (const sex of DATA.sexes) {
    if (!checkedSexes.has(sex)) continue;
    const src = stratum.by_sex[sex];
    if (!src) continue;
    cells = cells.map((c, i) => addCell(c, src[i]));
  }
  return cells;
}

function selectedStrata() {
  return DATA.strata
    .filter((s) => checkedStrata.has(s.name))
    .map((s) => ({ name: s.name, cells: cellsFor(s) }));
}

function selectedSexLabel() {
  const sexes = DATA.sexes.filter((s) => checkedSexes.has(s));
  if (sexes.length === 0) return "";
  return " · " + sexes.join("+");
}

const COLOR = { apparent: "#c0392b", corr: "#1e8449", grid: "#e3e7ec", text: "#5b6675" };
Chart.defaults.color = COLOR.text;
Chart.defaults.font.family = "system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif";
Chart.defaults.animation = false;
Chart.defaults.devicePixelRatio = 1;
Chart.defaults.resizeDelay = 80;
const ZOOM_PLUGIN = window.ChartZoom || window.chartjsPluginZoom || window["chartjs-plugin-zoom"];
if (ZOOM_PLUGIN) Chart.register(ZOOM_PLUGIN);

function fmtPct(v) { return v === null ? "n/a" : (v * 100).toFixed(1) + "%"; }
function fmtVeTooltip(v) { return v === null ? "n/a" : v.toFixed(1) + "%"; }

function veYBounds() {
  let min = parseFloat($("veYMin").value);
  let max = parseFloat($("veYMax").value);
  if (!Number.isFinite(min)) min = -100;
  if (!Number.isFinite(max)) max = 100;
  if (min === max) max = min + 1;
  return { min: Math.min(min, max), max: Math.max(min, max) };
}

function veYScale(title) {
  const { min, max } = veYBounds();
  return {
    min, max,
    title: { display: true, text: title },
    grid: { color: COLOR.grid },
    ticks: { callback: (v) => v + "%" },
  };
}

function clipVeValue(v) {
  if (v === null || !$("veYHideOut").checked) return v;
  const { min, max } = veYBounds();
  return v < min || v > max ? null : v;
}
function clipVeArray(arr) { return arr.map(clipVeValue); }

let barChart = null;
let timeCharts = [];
function destroyTimeCharts() {
  for (const c of timeCharts) c.destroy();
  timeCharts = [];
}

function bandRow(stratum, startIdx, endIdx) {
  const cell = sumRange(stratum, startIdx, endIdx);
  return { label: stratum.name, cell, apparent: apparentVE(cell), corr: correctedVE(cell), reliable: isReliable(cell) };
}

function renderChart(rows) {
  const showApparent = $("showApparent").checked;
  const showCorr = $("showCorr").checked;
  const labels = rows.map((r) => r.label + (r.reliable ? "" : " \u26a0"));

  const datasets = [];
  if (showApparent) {
    datasets.push({
      label: "Apparent VE",
      data: rows.map((r) => clipVeValue(r.apparent === null ? null : r.apparent * 100)),
      backgroundColor: rows.map((r) => (r.reliable ? COLOR.apparent : COLOR.apparent + "66")),
      borderRadius: 3,
    });
  }
  if (showCorr) {
    datasets.push({
      label: "Corrected VE",
      data: rows.map((r) => clipVeValue(r.corr === null ? null : r.corr * 100)),
      backgroundColor: rows.map((r) => (r.reliable ? COLOR.corr : COLOR.corr + "66")),
      borderRadius: 3,
    });
  }

  const cfg = {
    type: "bar",
    data: { labels, datasets },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: "index", intersect: false },
      scales: { y: veYScale("Effectiveness (%)"), x: { grid: { display: false } } },
      plugins: {
        legend: { labels: { usePointStyle: true } },
        tooltip: {
          callbacks: {
            label: (ctx) => {
              const r = rows[ctx.dataIndex];
              const raw = ctx.dataset.label === "Apparent VE"
                ? (r.apparent === null ? null : r.apparent * 100)
                : (r.corr === null ? null : r.corr * 100);
              return `${ctx.dataset.label}: ${fmtVeTooltip(raw)}`;
            },
            afterBody: (items) => {
              const r = rows[items[0].dataIndex];
              return [
                "",
                `target (exposed)      = ${r.cell.target_exposed}`,
                `reference (exposed)   = ${r.cell.reference_exposed}`,
                `target (unexposed)    = ${r.cell.target_unexposed}`,
                `reference (unexposed) = ${r.cell.reference_unexposed}`,
                r.reliable ? "" : "\u26a0 few events \u2014 unstable",
              ];
            },
          },
        },
      },
    },
  };

  if (barChart) barChart.destroy();
  barChart = new Chart($("barCanvas").getContext("2d"), cfg);
}

function renderTable(rows) {
  const tb = $("dataTable").querySelector("tbody");
  tb.innerHTML = "";
  for (const r of rows) {
    const tr = document.createElement("tr");
    if (!r.reliable) tr.className = "unreliable";
    const flag = r.reliable ? "" : ' <span class="flag" title="few events, unstable">&#9888;</span>';
    tr.innerHTML =
      `<td>${r.label}${flag}</td>` +
      `<td>${r.cell.target_exposed}</td><td>${r.cell.reference_exposed}</td>` +
      `<td>${r.cell.target_unexposed}</td><td>${r.cell.reference_unexposed}</td>` +
      `<td>${fmtPct(r.apparent)}</td><td>${fmtPct(r.corr)}</td>`;
    tb.appendChild(tr);
  }
}

function timeSeries(stratum, startIdx, endIdx) {
  const labels = [];
  const apparent = [];
  const corr = [];
  const counts = [];
  const reliablePt = [];
  for (let i = startIdx; i <= endIdx; i++) {
    const cell = stratum.cells[i];
    const av = apparentVE(cell);
    const cv = correctedVE(cell);
    labels.push(DATA.periods[i]);
    apparent.push(av === null ? null : av * 100);
    corr.push(cv === null ? null : cv * 100);
    counts.push(cell);
    reliablePt.push(isReliable(cell));
  }
  return { labels, apparent, corr, counts, reliablePt };
}

function makeLineChart(canvas, ts) {
  const showApparent = $("showApparent").checked;
  const showCorr = $("showCorr").checked;

  const pointStyle = (key) => ({
    radius: ts.reliablePt.map((r) => (r ? 3 : 2)),
    pointBackgroundColor: ts.reliablePt.map((r) => (r ? COLOR[key] : COLOR[key] + "55")),
  });

  const datasets = [];
  if (showApparent) {
    datasets.push({
      label: "Apparent VE",
      data: clipVeArray(ts.apparent),
      borderColor: COLOR.apparent,
      backgroundColor: COLOR.apparent,
      tension: 0.2,
      spanGaps: true,
      ...pointStyle("apparent"),
    });
  }
  if (showCorr) {
    datasets.push({
      label: "Corrected VE",
      data: clipVeArray(ts.corr),
      borderColor: COLOR.corr,
      backgroundColor: COLOR.corr,
      tension: 0.2,
      spanGaps: true,
      ...pointStyle("corr"),
    });
  }

  const cfg = {
    type: "line",
    data: { labels: ts.labels, datasets },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: "index", intersect: false },
      scales: {
        y: veYScale("VE (%)"),
        x: { grid: { color: COLOR.grid }, ticks: { maxRotation: 0, autoSkip: true, maxTicksLimit: 7 } },
      },
      plugins: {
        legend: { labels: { usePointStyle: true, boxWidth: 8 } },
        tooltip: {
          callbacks: {
            label: (ctx) => {
              const raw = ctx.dataset.label === "Apparent VE" ? ts.apparent[ctx.dataIndex] : ts.corr[ctx.dataIndex];
              return `${ctx.dataset.label}: ${fmtVeTooltip(raw)}`;
            },
            afterBody: (items) => {
              const c = ts.counts[items[0].dataIndex];
              const rel = ts.reliablePt[items[0].dataIndex];
              return [
                "",
                `target: exposed=${c.target_exposed} unexposed=${c.target_unexposed}`,
                `reference: exposed=${c.reference_exposed} unexposed=${c.reference_unexposed}`,
                rel ? "" : "\u26a0 few events \u2014 unstable",
              ];
            },
          },
        },
        zoom: {
          zoom: {
            wheel: { enabled: false },
            pinch: { enabled: false },
            drag: { enabled: true, backgroundColor: "rgba(36,113,163,0.15)", borderColor: "#2471a3", borderWidth: 1 },
            mode: "x",
          },
        },
      },
    },
  };
  return new Chart(canvas.getContext("2d"), cfg);
}

function renderTimeView(strata, startIdx, endIdx) {
  const grid = $("timeGrid");
  destroyTimeCharts();
  grid.innerHTML = "";
  $("timeHint").innerHTML =
    `Each point is one CSV period. Hover for exact values; ` +
    `drag to select a zoom region (X axis), "Reset zoom" to restore. Faded points = few events (unstable).`;
  if (strata.length === 0 || checkedSexes.size === 0) {
    grid.innerHTML = `<p class="hint">Select at least one stratum and one sex.</p>`;
    return;
  }
  for (const s of strata) {
    const cell = sumRange(s, startIdx, endIdx);
    const cellEl = document.createElement("div");
    cellEl.className = "time-cell";
    const head = document.createElement("h3");
    head.textContent = s.name;
    const meta = document.createElement("p");
    meta.className = "meta";
    meta.textContent =
      `range total: target exposed=${cell.target_exposed} unexposed=${cell.target_unexposed}, ` +
      `reference exposed=${cell.reference_exposed} unexposed=${cell.reference_unexposed}`;
    const wrap = document.createElement("div");
    wrap.className = "canvas-wrap";
    const canvas = document.createElement("canvas");
    wrap.appendChild(canvas);
    cellEl.append(head, meta, wrap);
    grid.appendChild(cellEl);

    const ts = timeSeries(s, startIdx, endIdx);
    timeCharts.push(makeLineChart(canvas, ts));
  }
}

function render() {
  const startIdx = DATA.periods.indexOf($("startPeriod").value);
  let endIdx = DATA.periods.indexOf($("endPeriod").value);
  if (endIdx < startIdx) endIdx = startIdx;

  const strata = selectedStrata();
  const timeMode = $("viewTime").checked;
  $("barsCard").hidden = timeMode;
  $("timeCard").hidden = !timeMode;

  const win = `${$("startPeriod").value} \u2192 ${$("endPeriod").value}${selectedSexLabel()}`;

  if (timeMode) {
    $("timeTitle").textContent = `VE over time by stratum  (${win})`;
    renderTimeView(strata, startIdx, endIdx);
    renderTable(strata.map((s) => bandRow(s, startIdx, endIdx)));
    return;
  }

  destroyTimeCharts();
  const rows = strata.map((s) => bandRow(s, startIdx, endIdx));
  $("chartTitle").textContent = `Estimated VE by stratum  (${win})`;
  renderChart(rows);
  renderTable(rows);
}

let appViewInitialized = false;

function initAppView() {
  fillPeriodSelects();
  renderSexList();
  renderStratumList();

  if (!appViewInitialized) {
    appViewInitialized = true;
    ["startPeriod", "endPeriod", "showApparent", "showCorr", "viewBars", "viewTime", "veYMin", "veYMax", "veYHideOut"].forEach((id) =>
      $(id).addEventListener("change", render)
    );
    $("selAll").addEventListener("click", () => {
      checkedStrata = new Set(DATA.strata.map((s) => s.name));
      renderStratumList(); render();
    });
    $("selNone").addEventListener("click", () => {
      checkedStrata = new Set();
      renderStratumList(); render();
    });
    $("resetZoom").addEventListener("click", () => {
      for (const c of timeCharts) c.resetZoom();
    });
  }

  render();
}

initUploadView();
