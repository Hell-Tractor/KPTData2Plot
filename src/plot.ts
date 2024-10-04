import { PlotData } from "./App.vue";
import * as echarts from 'echarts';

export async function draw_plot(data: PlotData) : Promise<void> {
    if (data.chart == null) {
        let chartDom = document.getElementById(`plot-${data.title}`);
        // console.log(chartDom);
        data.chart = echarts.init(chartDom);
    }

    let series = data.curves.flatMap((curve) => {
        return [{
            data: curve.data.map((x) => x.avg - x.se),
            type: 'line',
            lineStyle: {
                opacity: 0
            },
            name: '__L' + curve.name,
            symbol: 'none',
            stack: '__S' + curve.name,
        }, {
            data: curve.data.map((x) => x.se * 2),
            type: 'line',
            lineStyle: {
                opacity: 0
            },
            areaStyle: {
                color: curve.fillColor
            },
            name: '__U' + curve.name,
            symbol: 'none',
            stack: '__S' + curve.name,
        }, {
            data: curve.data.map((x) => x.avg),
            type: 'line',
            name: curve.name,
            itemStyle: {
                color: curve.lineColor
            },
            symbol: 'none',
        }]
    })
    console.log(series);
    if (data.xStep < 2)
        data.xStep = 2;

    data.chart.setOption({
        title: {
            text: data.title,
            left: 'center'
        },
        xAxis: {
            type: 'category',
            axisLabel: {
                interval: data.xStep - 1
            }
        },
        yAxis: {
            type: 'value',
            interval: data.yStep
        },
        series: series
    })
}