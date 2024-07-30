<script setup lang="ts">
import { Ref } from 'vue';
import { CurveData, PlotData } from '../App.vue';
import { rules } from '../Rules';

defineProps<{ curves: CurveData[] }>();

const plotDatas: Ref<PlotData[]> = defineModel({ type: Array<PlotData>, required: true });

function add_empty_plot() : void {
  plotDatas.value.push({
    title: "",
    curves: [],
    chart: null
  });
}
</script>

<template>
  <v-table>
    <thead>
      <tr>
        <th style="width: 40%;">Title</th>
        <th style="width: 60%;">Curves</th>
        <th style="width: 30px;">Delete</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(plotData, index) in plotDatas" :key="index">
        <td>
          <v-text-field class="padding-top" v-model="plotData.title" density="compact" :rules="[rules.required]"></v-text-field>
        </td>
        <td>
          <v-select class="padding-top" v-model="plotData.curves" :items="curves" item-title="name" return-object multiple density="compact" chips></v-select>
        </td>
        <td>
          <v-btn icon="mdi-delete" @click="plotDatas.splice(index, 1)" variant="flat"></v-btn>
        </td>
      </tr>
    </tbody>
  </v-table>
  <v-card-actions>
    <v-spacer></v-spacer>
    <v-btn icon="mdi-plus" @click="add_empty_plot"></v-btn>
  </v-card-actions>
</template>

<style scoped>
.padding-top {
  padding-top: 15px;
}
</style>