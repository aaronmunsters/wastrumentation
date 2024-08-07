import pandas as pd
runtime_analysis = pd.read_csv('runtime-analysis.csv', header=0)
runtime_analysis_wastrumentation = pd.read_csv('runtime-analysis-wastrumentation.csv', header=0)

runtime_analysis = runtime_analysis.groupby(["benchmark", "runtime_environment"]).median().reset_index()
runtime_analysis_wastrumentation = runtime_analysis_wastrumentation.groupby(["benchmark", "runtime_environment"]).median().reset_index()

runtime_analysis.rename(columns={"performance": "performance_baseline"}, inplace=True)
runtime_analysis['wastrumentation_performance'] = runtime_analysis_wastrumentation['performance']
runtime_analysis['slowdown (x)'] = runtime_analysis.apply(lambda x: x['wastrumentation_performance'] / x['performance_baseline'], axis=1)

# report results
print(runtime_analysis)
