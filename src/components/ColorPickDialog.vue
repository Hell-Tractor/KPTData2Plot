<script setup lang="ts">
import { Ref, ref } from 'vue';

const model_color = defineModel<string>({ type: String, required: true });

const selected_color: Ref<string> = ref(model_color.value);
</script>

<template>
    <v-dialog max-width="60%">
        <template v-slot:activator="{ props }">
            <slot name="activator" :props="props"></slot>
        </template>
        <template v-slot:default="{ isActive }">
            <v-card>
                <v-card-text>
                    <v-color-picker elevation="0" width="100%" v-model="selected_color" mode="rgb"></v-color-picker>
                </v-card-text>
                <v-card-actions>
                    <v-spacer></v-spacer>
                    <v-btn @click="isActive.value = false; model_color = selected_color">确定</v-btn>
                    <v-btn @click="isActive.value = false">取消</v-btn>
                </v-card-actions>
            </v-card>
        </template>
    </v-dialog>
</template>