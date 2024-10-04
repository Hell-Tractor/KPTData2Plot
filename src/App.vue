<script setup lang="ts">
import { Ref, ref } from 'vue';
import { open } from '@tauri-apps/api/dialog';
import { invoke } from '@tauri-apps/api';
import CurveConfigPanel from './components/CurveConfigPanel.vue';
import PlotConfigPanel from './components/PlotConfigPanel.vue';
import { draw_plot } from './plot';

export interface CurveData {
  name: string;
  sourceName: string;
  lineColor: string;
  fillColor: string;
  unit: number;
  maxLength: number;
  data: {
    avg: number;
    se: number;
  }[];
}

export interface PlotData {
  title: string;
  curves: CurveData[];
  chart: echarts.ECharts | null;
  xStep: number;
  yStep: number;
}

const filePath: Ref<string | null> = ref(null);
const isFileProcessing: Ref<boolean> = ref(false);
const alertMessage: Ref<string | null> = ref(null);
const excelHeaders: Ref<string[]> = ref([]);
const plotDatas: Ref<PlotData[]> = ref([]);
const curves: Ref<CurveData[]> = ref([]);
const drawToolTip: Ref<string> = ref("");
const isDrawing: Ref<boolean> = ref(false);
const showPlot: Ref<boolean> = ref(false);

function reset() : void {
  excelHeaders.value = [];
  plotDatas.value = [];
  curves.value = [];
  showPlot.value = false;
}

async function select_file() : Promise<void> {
  var newPath = await open({
    multiple: false,
    filters: [{ name: 'CSV File', extensions: ['csv'] }]
  }) as string | null;
  if (newPath !== null && newPath !== filePath.value) {
    filePath.value = newPath;
    reset();
  }
}

async function process() : Promise<void> {
  if (filePath.value === null) {
    alertMessage.value = "Please select a file before processing.";
    return;
  }
  isFileProcessing.value = true;
  invoke("get_excel_header", { path: filePath.value })
    .then((result) => {
      excelHeaders.value = result as string[]
      if (excelHeaders.value.length === 0) {
        alertMessage.value = "No data found in the file.";
      }
    })
    .catch((error: Error) => {
      alertMessage.value = error.toString();
    }).finally(() => {
      isFileProcessing.value = false;
    });
}

async function draw() : Promise<void> {
  isDrawing.value = true;
  let curves = plotDatas.value.flatMap((plotData) => plotData.curves).filter((curve, index, self) => self.indexOf(curve) === index);
  let configs = curves.map((curve) => {
    return {
      column_id: excelHeaders.value.indexOf(curve.sourceName),
      unit: curve.unit,
      max_length: curve.maxLength,
    };
  })
  let ids = curves.map((curve) => excelHeaders.value.indexOf(curve.sourceName));
  console.log("curves", curves);
  console.log("ids", ids);
  console.log("configs", configs);
  invoke("get_data", { path: filePath.value, curveConfigs: configs }).then((result) => {
    let datas = result as { avg: number, se: number }[][];
    console.log("result", result);
    showPlot.value = true;
    for (let plotData of plotDatas.value) {
      for (let curve of plotData.curves) {
        curve.data = datas[ids.indexOf(excelHeaders.value.indexOf(curve.sourceName))];
      }
      draw_plot(plotData);
    }
  }).catch((error) => {
    alertMessage.value = error.toString();
  }).finally(() => {
    isDrawing.value = false;
  });
}

function allow_draw() : boolean {
  if (plotDatas.value.length == 0) {
    drawToolTip.value = "No plot created.";
    return false;
  }

  for (let i = 0; i < plotDatas.value.length; i++) {
    let plotData = plotDatas.value[i];
    if (plotData.title == "") {
      drawToolTip.value = `Plot title with index ${i} is empty.`;
      return false;
    }
    if (plotData.curves.length == 0) {
      drawToolTip.value = `Plot with title '${plotData.title}' has no curve.`;
      return false;
    }
    for (let j = 0; j < plotData.curves.length; j++) {
      let curve = plotData.curves[j];
      if (curve.name == "") {
        drawToolTip.value = `Curve name with index ${j} in plot '${plotData.title}' is empty.`;
        return false;
      }
      if (curve.sourceName == "") {
        drawToolTip.value = `Data source in curve '${curve.name}' is empty.`;
        return false;
      }
    }
  }

  for (let curve of curves.value) {
    if (curves.value.find((c) => c.name === curve.name) !== curve) {
      drawToolTip.value = `Curve with name '${curve.name}' is duplicated.`;
      return false;
    }
  }

  for (let plotData of plotDatas.value) {
    if (plotDatas.value.find((p) => p.title === plotData.title) !== plotData) {
      drawToolTip.value = `Plot with title '${plotData.title}' is duplicated.`;
      return false;
    }
  }

  drawToolTip.value = `${plotDatas.value.length} plot(s) will be drawn.`;
  return true;
}

function remove_curve_from_plots(curve: CurveData) {
  for (let plotData of plotDatas.value) {
    plotData.curves = plotData.curves.filter((c) => c !== curve);
  }
}

async function export_plots() : Promise<void> {
  var output_path = await open({
    directory: true,
    multiple: false
  }) as string | null;
  if (output_path == null)
    return;

  plotDatas.value.map((data) => [
    data.title,
    data.chart?.getDataURL({
      type: 'png',
      backgroundColor: '#fff'
    })
  ]).filter((data) => data[1] != null).forEach((data) => {
    let path = `${output_path}/${data[0]}.png`;
    console.log("save image to " + path);
    invoke("save_image", { path: path, image: data[1] })
      .catch((error) => {
        alertMessage.value = error.toString();
      });
  });
}
</script>

<template>
  <v-alert color="error" v-if="alertMessage" icon="$error" closable @click:close="alertMessage = null">
    {{ alertMessage }}
  </v-alert>
  <v-text-field placeHolder="Select Excel File" v-model="filePath" prepend-icon="mdi-paperclip" readonly @click="select_file" :disabled="isFileProcessing">
    <template #append>
      <v-btn @click.stop="process" :loading="isFileProcessing" :disabled="filePath === null || excelHeaders.length > 0">
        Process
      </v-btn>
    </template>
  </v-text-field>
  <v-expansion-panels>
    <v-expansion-panel title="Curves" v-if="excelHeaders.length > 0">
      <v-expansion-panel-text>
        <CurveConfigPanel :data-sources="excelHeaders" v-model="curves" @remove_curve="remove_curve_from_plots"></CurveConfigPanel>
      </v-expansion-panel-text>
    </v-expansion-panel>
    <v-expansion-panel title="Plot" v-if="curves.length > 0">
      <v-expansion-panel-text>
        <PlotConfigPanel :curves="curves" v-model="plotDatas"></PlotConfigPanel>
      </v-expansion-panel-text>
    </v-expansion-panel>
  </v-expansion-panels>
  <v-tooltip :text="drawToolTip" v-if="plotDatas.length > 0" location="bottom">
    <template v-slot:activator="{ props }">
      <div v-bind="props">
        <v-btn @click="draw" :disabled="!allow_draw()" block color="primary" :loading="isDrawing">draw!</v-btn>
      </div>
    </template>
  </v-tooltip>
  <v-btn @click="export_plots" :disabled="!showPlot" block color="success" v-if="plotDatas.length > 0">Export All</v-btn>
  <div class="plot" v-for="data in plotDatas" :id="`plot-${data.title}`"></div>
</template>

<style scoped>
.plot {
  width: 600px;
  height: 400px;
}
</style>
