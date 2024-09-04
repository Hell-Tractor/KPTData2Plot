<script setup lang="ts">
import { ref, Ref } from 'vue';
import { CurveData } from '../App.vue';
import ColorPickDialog from './ColorPickDialog.vue';
import { rules } from '../Rules';

const props = defineProps<{ dataSources: string[] }>();

const curves: Ref<CurveData[]> = defineModel({ type: Array<CurveData>, required: true });

const emits = defineEmits<{
  remove_curve: [CurveData]
}>();

const dataSourceFilter: Ref<string> = ref("");

function get_random_color() : string {
  return "#" + Math.floor(Math.random()*16777215).toString(16).padStart(6, "0");
}

function add_empty_curve() : void {
  curves.value.push({
    name: "",
    sourceName: "",
    lineColor: get_random_color(),
    fillColor: get_random_color(),
    data: []
  });
}

function get_filtered_data_sources() : string[] {
  if (dataSourceFilter.value === "") {
    return props.dataSources;
  }
  var keywords = dataSourceFilter.value.toLowerCase().split(" ");
  return props.dataSources.filter((dataSource) => keywords.every((keyword) => dataSource.toLowerCase().includes(keyword)));
}
</script>

<template>
  <v-table>
    <thead>
      <tr>
        <th style="width: 50%;">Curve Name</th>
        <th style="width: 50%;">Data Source</th>
        <th style="width: 50px;">Line Color</th>
        <th style="width: 50px;">Fill Color</th>
        <th style="width: 30px;">Delete</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(curve, index) in curves" :key="index">
        <td>
          <v-text-field class="padding-top" v-model="curve.name" density="compact" :rules="[rules.required]"></v-text-field>
        </td>
        <td>
          <v-select class="padding-top" v-model="curve.sourceName" :items="get_filtered_data_sources()" density="compact">
            <template v-slot:prepend-item>
              <v-text-field placeholder="Filter" v-model="dataSourceFilter"></v-text-field>
            </template>
          </v-select>
        </td>
        <td>
          <ColorPickDialog v-model="curve.lineColor">
            <template v-slot:activator="{ props: lineColorPickerActivator}">
              <v-btn v-bind="lineColorPickerActivator" :style="{ backgroundColor: curve.lineColor }"></v-btn>
            </template>
          </ColorPickDialog>
        </td>
        <td>
          <ColorPickDialog v-model="curve.fillColor">
            <template v-slot:activator="{ props: fillColorPickerActivator}">
              <v-btn v-bind="fillColorPickerActivator" :style="{ backgroundColor: curve.fillColor }"></v-btn>
            </template>
          </ColorPickDialog>
        </td>
        <td>
          <v-btn icon="mdi-delete" @click="emits('remove_curve', curves[index]); curves.splice(index, 1)" variant="flat"></v-btn>
        </td>
      </tr>
    </tbody>
  </v-table>
  <v-card-actions>
    <v-spacer></v-spacer>
    <v-btn icon="mdi-plus" @click="add_empty_curve"></v-btn>
  </v-card-actions>
</template>

<style scoped>
.padding-top {
  padding-top: 15px;
}
</style>