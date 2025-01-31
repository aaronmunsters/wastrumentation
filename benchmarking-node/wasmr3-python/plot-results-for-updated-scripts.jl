#---
# CSVFiles, VegaLite, DataFrames
using Statistics, Printf
using VegaLite, CSVFiles, DataFrames
#---
target_dir_prefix = "./working-dir/dragonfly-25-01-07/"
#---
# Compatibility
## Read in `.csv` data files from what works and what does not
#---
function parse_boolean(s::String)
	if s === "False"
		return false
	elseif s === "True"
		return true
	else
		error("Cannot parse $s as boolean")
	end
end

function parse_booleans(df)
	df = transform(df, :exception => ByRow(parse_boolean) => :exception)
	df = transform(df, :timeout => ByRow(parse_boolean) => :timeout)
	df
end

exec_once = DataFrame(load(target_dir_prefix * "executes-once.csv")) |> parse_booleans
#---
total_input_programs = length(groupby(exec_once, :input_program)) # Total number of input programs

regular_success = nrow(filter([:platform, :completion_time] => (p, c) -> p === "uninstrumented" && c !== missing, exec_once))
regular_timeout = nrow(filter([:platform, :timeout] => (p, t) -> p === "uninstrumented" && t, exec_once))
regular_timeout_report = if regular_timeout == 0
	begin
		""
	end
else
	" ($regular_timeout timed out)"
end

wasabi_success = nrow(filter([:platform, :completion_time] => (p, c) -> p === "Wasabi" && c !== missing, exec_once))
wasabi_timeout = nrow(filter([:platform, :timeout] => (p, t) -> p === "Wasabi" && t, exec_once))
wasabi_error_r = nrow(filter([:platform, :exception] => (p, e) -> p === "Wasabi" && e, exec_once))
wasabi_unsuccesful_report = if wasabi_timeout == 0 && wasabi_error_r == 0
	begin
		""
	end
elseif wasabi_timeout == 0 && wasabi_error_r > 0
	begin
		" ($wasabi_error_r errored)"
	end
elseif wasabi_timeout > 0 && wasabi_error_r == 0
	begin
		" ($wasabi_timeout timed out)"
	end
else
	" ($wasabi_timeout timed out, $wasabi_error_r errored)"
end

wastrm_success = nrow(filter([:platform, :completion_time] => (p, c) -> p === "Wastrumentation" && c !== missing, exec_once))
wastrm_timeout = nrow(filter([:platform, :timeout] => (p, t) -> p === "Wastrumentation" && t, exec_once))
wastrm_error_r = nrow(filter([:platform, :exception] => (p, e) -> p === "Wastrumentation" && e, exec_once))

wastrm_timeout_report = if wastrm_timeout == 0
	begin
		""
	end
else
	" ($wastrm_timeout timed out)"
end

conclusion = "For the forward analysis on a total of $total_input_programs input programs,
our benchmark harness succesfully executed $regular_success programs uninstrumented$regular_timeout_report,
$wastrm_success after instrumentation by Wastrumentation$wastrm_timeout_report and
$wasabi_success after instrumentation by Wasabi$wasabi_unsuccesful_report."

println(conclusion)
#---
# Memory Usage Study:
#---
exec_per_platform = groupby(exec_once, :platform)
exec_once_wastrm = exec_per_platform[("Wastrumentation",)]
exec_once_wasabi = exec_per_platform[("Wasabi",)]
exec_once_rgular = exec_per_platform[("uninstrumented",)]

exec_once_baseline = rename(
	select(exec_once_rgular, [:input_program, :memory_usage, :exception, :timeout]),
	:memory_usage => :baseline_memory_usage,
	:exception => :baseline_exception,
	:timeout => :baseline_timeout,
)

wasabi_wastrm_mem_usage = vcat(exec_once_wastrm, exec_once_wasabi)
wasabi_wastrm_mem_overhead = transform(
	innerjoin(exec_once_baseline, wasabi_wastrm_mem_usage, on = [:input_program]),
	[:baseline_memory_usage, :memory_usage] =>
		ByRow((baseline_memory_usage, memory_usage) -> begin
			if memory_usage === missing || baseline_memory_usage === missing
				missing
			else
				memory_usage / baseline_memory_usage
			end
		end)
		=> :memory_usage_overhead,
)

"Parsed memory data frames"
#---
wasabi_wastrm_mem_overhead_plot =
	wasabi_wastrm_mem_overhead |> @vlplot(
		encoding = {
			x = {
				field = "input_program",
				type = "nominal",
				axis = {
					labelAngle = -30,
					title = "Input Program",
					labelFontSize = 10,
					titleFontSize = 12,
					titlePadding = 10,
				},
			},
			y = {
				field = "memory_usage_overhead",
				type = "quantitative",
				axis = {
					title = "Runtime Memory Overhead (X)",
					titleFontSize = 12,
					labelFontSize = 10,
					grid = false,
				},
				scale = {
					type = "log",
				},
			},
			color = {
				field = "platform",
				type = "nominal",
				scale = {
					scheme = "pastel2",
				},
				legend = {
					title = "Instrumentation Platform",
					orient = "top",
					titleFontSize = 12,
					labelFontSize = 10,
				},
			},
			xOffset = {
				field = "platform",
				type = "nominal",
			},
			size = {value = 10},  # Adjusts the width of the bars
		},
		layer = [
			{
				mark = {
					type = "bar",
					cornerRadiusEnd = 3,
				},
			},
			{
				mark = {
					type = "text",
					align = "center",
					baseline = "middle",
					angle = 90,  # Rotates the labels vertically
					fontSize = 10,
					dx = -15  # Adjust horizontal offset to center labels over bars
				},
				encoding = {
					text = {
						field = "memory_usage_overhead",
						type = "quantitative",
						format = ".2f"  # Adjust number format as needed
					},
				},
			},
		],
		title = "Memory Overhead Comparison for Forward Analysis using Wasabi and Wastrumentation",
		config = {
			view = {stroke = :transparent},
			axis = {
				domainColor = "#999",
			},
		},
		width = 550,
		height = 100,
	)

wasabi_wastrm_mem_overhead_plot |> save(target_dir_prefix * "memory-overhead.pdf")
wasabi_wastrm_mem_overhead_plot
#---
# Code Size Study
#---
df_code_sizes = DataFrame(load(target_dir_prefix * "code-sizes.csv"))
"Data files read"
#---
# Plot Code Size Increase
#---
baseline_code_size = select(filter(row -> row.platform .== "uninstrumented", df_code_sizes), Not(:analysis, :platform))
baseline_code_size_renamed = rename(baseline_code_size, :size_bytes => :size_bytes_baseline)

code_size_inc_transformation = [:size_bytes, :size_bytes_baseline] => ((size_bytes, size_bytes_baseline) -> size_bytes ./ size_bytes_baseline) => :code_increase
code_size_baseline_joined = leftjoin(
	filter(row -> row.platform !== "uninstrumented", df_code_sizes),
	baseline_code_size_renamed,
	on = :input_program,
)

code_size_overhead = transform(code_size_baseline_joined, code_size_inc_transformation)
"Relative code size increase computed"
#---
absolute_code_size_plot =
	baseline_code_size_renamed |> @vlplot(
		encoding = {
			x = {
				field = "input_program",
				type = "nominal",
				axis = {title = "Input Program"},
				axis = {labelAngle = "-30"},
			},
			y = {
				field = "size_bytes_baseline",
				type = "quantitative",
				axis = {
					title = "Program Size (bytes)",
					grid = false,
					titleFontSize = 12,
					labelFontSize = 10,
					labelColor = "#666",  # Softer axis label color
					domainColor = "#999"  # Softer axis line color
				},
				scale = {
					type = "log",
				},
			},
			color = {
				value = "#4C72B0"  # Soft blue color for bars
			},
		},
		layer = [
			{
				mark = {
					type = "bar",
					cornerRadiusEnd = 3  # Adds rounded corners to the top of bars
				},
			},
			{
				mark = {
					type = "text",
					align = "center",
					baseline = "middle",
					fontSize = 10,
					dx = -27,
				},
				encoding = {
					text = {
						field = "size_bytes_baseline",
						type = "quantitative",
						format = ",.0f",
					},
					angle = {"value" = 90},
				},
			},
		],
		config = {
			view = {stroke = :transparent},
		},
		title = "Absolute Code Size per Input Program",  # Adds a descriptive title
		width = 550,
		height = 100,
	)
absolute_code_size_plot |> save(target_dir_prefix * "absolute-code-size-plot.pdf")
absolute_code_size_plot
#---
code_incr_forward = filter(
	:analysis => .==("forward"),
	filter(row -> row.analysis !== missing, code_size_overhead),
)
binary_size_plot =
	code_incr_forward |>
	@vlplot(
		width = 500,
		layer = [
			{
				mark = "bar",
				encoding = {
					color = {
						field = "platform",
						type = "nominal",
						legend = {
							title = "Instrumentation Platform",
							orient = "top",
						},
					},
					xOffset = {
						field = "platform",
						type = "nominal",
					},
					x = {
						field = "input_program",
						type = "nominal",
						axis = {labelAngle = "45"},
						title = "Input Program",
					},
					y = {
						field = "code_increase",
						type = "quantitative",
						axis = {
							title = "Program Size Increase (X)",
							grid = false,
						},
					},
				},
			},
			{
				mark = "rule",
				encoding = {
					y = {
						datum = 1,
					},
					color = {value = "red"}, # Color for the line
					size = {value = 1} # Thickness of the line
				},
			},
		],
		config = {
			view = {stroke = :transparent},
		},
	)

binary_size_plot |> save(target_dir_prefix * "wasabi-wastrm-binary-size.pdf")
binary_size_plot
#---
all_code_sizes_plot =
	code_size_overhead |> @vlplot(
		"transform" = [
			{
				"calculate" = "substring(datum.code_increase, 0, 5)",
				"as" = "code_increase_truncated",
			},
		],
		facet = {
			row = {
				field = "platform",
				type = "nominal",
			},
		},
		spec = {
			layer = [
				{
					mark = "rect",
					encoding = {
						y = {
							field = "analysis",
							type = "nominal",
							axis = {labelAngle = "-30"},
						},
						x = {
							field = "input_program",
							type = "nominal",
							axis = {title = "Input Program"},
							axis = {labelAngle = "-30"},
						},
						color = {
							field = "code_increase",
							type = "quantitative",
							scale = {
								type = "log",
								scheme = "blues",
							},
							legend = {
								title = "Code Size Increase (X)",
								orient = "top",
								titleLimit = 1000,
								gradientLength = 540,
								titleAnchor = "center",
								titleAlign = "center",
							},
						},
					},},
				{
					mark = {
						type = "text",
						fontSize = "6",
					},
					encoding = {
						y = {
							field = "analysis",
							type = "nominal",
						},
						x = {
							field = "input_program",
							type = "nominal",
							axis = {title = "Input Program"},
						},
						text = {
							field = "code_increase_truncated",
							type = "quantitative",
						},
					},
				},
			],},
		config = {
			axis = {
				grid = true,
				tickBand = "extent",
			},
		},
	)

all_code_sizes_plot |> save(target_dir_prefix * "wasabi-wastrm-binary-size.pdf")
all_code_sizes_plot
#---
wasabi_code_size = select(filter(row -> row.platform .== "Wasabi", df_code_sizes), Not(:platform))
wastrm_code_size = select(filter(row -> row.platform .== "Wastrumentation", df_code_sizes), Not(:platform))
"Absolute code sizes computed"
#---
wasabi_wastrm_size_overhead = transform(
	outerjoin(
		rename(wasabi_code_size, :size_bytes => :wasabi_size_bytes),
		rename(wastrm_code_size, :size_bytes => :wastrm_size_bytes),
		on = [:analysis, :input_program],
	),
	[:wasabi_size_bytes, :wastrm_size_bytes]
	=> ((wasabi_size_bytes, wastrm_size_bytes) -> wasabi_size_bytes ./ wastrm_size_bytes)
		=> :code_size_for_wasabi_over_wastrm,
)
"Relative code size overhead computed"
#---
@assert length(groupby(wastrm_code_size, :input_program)) == 27
length(groupby(wastrm_code_size, :input_program)) # FIXME: ???
#---
code_size_for_wasabi_per_time_for_wastrm_non_missing = filter(row -> row.code_size_for_wasabi_over_wastrm !== missing, wasabi_wastrm_size_overhead)
total_outout_cpmr_progms = nrow(code_size_for_wasabi_per_time_for_wastrm_non_missing)
wastrm_output_is_smaller = filter(row -> row.code_size_for_wasabi_over_wastrm > 1, code_size_for_wasabi_per_time_for_wastrm_non_missing)
wasabi_output_is_smaller = filter(row -> row.code_size_for_wasabi_over_wastrm < 1, code_size_for_wasabi_per_time_for_wastrm_non_missing)
wastrm_wasabi_equal_size = filter(row -> row.code_size_for_wasabi_over_wastrm === 1, code_size_for_wasabi_per_time_for_wastrm_non_missing)


sentence = "For a total of $total_outout_cpmr_progms output programs, $(nrow(wastrm_output_is_smaller)) have a smaller output size for Wastrumentation and $(nrow(wasabi_output_is_smaller)) have a smaller output size for Wasabi."
sentence
# mean(code_size_for_wasabi_per_time_for_wastrm_non_missing.code_size_for_wasabi_over_wastrm)
# std(code_size_for_wasabi_per_time_for_wastrm_non_missing.code_size_for_wasabi_over_wastrm, corrected=false)
#---
# transform(wasabi_wastrm_size_overhead,
#   [:code_size_for_wasabi_over_wastrm] => ByRow((r) -> if r == )
# )
skipmissing(wasabi_wastrm_size_overhead)
#---
binary_size_overhead_comparison_plot =
	wasabi_wastrm_size_overhead |>
	@vlplot(
		"transform" = [
			{
				# Calculate truncated value and replace null with an empty string
				"calculate" = "datum.code_size_for_wasabi_over_wastrm == null ? '' : substring(datum.code_size_for_wasabi_over_wastrm, 0, 5)",
				"as" = "relative_overhead_truncated",
			},
		],
		encoding = {
			y = {
				field = "analysis",
				type = "nominal",
				axis = {labelAngle = "-30"},
			},
			x = {
				field = "input_program",
				type = "nominal",
				axis = {title = "Input Program", labelAngle = "-30"},
			},
		},
		layer = [
			{
				mark = "rect",
				encoding = {
					color = {
						field = "code_size_for_wasabi_over_wastrm",
						type = "quantitative",
						scale = {
							domainMid = 1,
							scheme = "redyellowgreen",
							type = "log",
						},
						legend = {
							title = "Wasabi Output Code Size / Wastrumentation Output Code Size",
							orient = "top",
							titleLimit = 1000,
							gradientLength = 540,
							titleAnchor = "center",
							titleAlign = "center",
						},
					},
				},
			},
			{
				mark = {
					type = "text",
					fontSize = "6",
				},
				encoding = {
					text = {
						field = "relative_overhead_truncated",
						type = "nominal",
					},
				},
			},
		],
		config = {
			axis = {grid = true, tickBand = "extent"},
		},
	)

binary_size_overhead_comparison_plot |> save(target_dir_prefix * "wastrumentation-wasabi-size-overhead-comparison.pdf")
binary_size_overhead_comparison_plot
#---
# String   : runtime
# String   : platform
# String   : analysis
# String   : input_program
# Int64?   : memory_usage
# Float64? : completion_time
# Int64    : runtime_iteration
# "ms"     : time_unit
# Bool     : exception
# String?  : exception_reason
# Bool     : timeout
# Int64    : timeout_amount

function parse_boolean(df, column)
	return transform(df, column => ByRow(parse_boolean) => column)
end

runtime_performance = DataFrame(load(target_dir_prefix * "executes-bench.csv"))
runtime_performance = parse_boolean(runtime_performance, :exception)
runtime_performance = parse_boolean(runtime_performance, :timeout)
runtime_performance

df_rgular = select(filter(row -> row.platform .== "uninstrumented", runtime_performance), Not([:platform, :analysis]))
df_wasabi = select(filter(row -> row.platform .== "Wasabi", runtime_performance), Not(:platform))
df_wastrm = select(filter(row -> row.platform .== "Wastrumentation", runtime_performance), Not(:platform))

# Squash together inter-runtime iterations => [1, 2, 3, 1, 2, 3] => [1, 2, 3]

function squash_inter_runtime_iterations(df)
	combine(
		groupby(df, Cols(:input_program, :runtime, :runtime_iteration, :platform, :analysis)),
		# :performance::Vec{Float64}, :error::Vec{Bool}, :timeout::Vec{Bool}
		[:performance, :error, :timeout] =>
		# combine rows such that:
		#   if any row has error, the combination is error
		#   if none are error, but any row is timeout, the combination is timeout
		#   none should be error or timeout, the combination is the median
			((perf, err, to) -> if any(Bool.(err))
				(; performance = missing, error = true, timeout = false)  # Return NamedTuple
			elseif any(Bool.(to))
				(; performance = missing, error = false, timeout = true)  # Return NamedTuple
			else
				(; performance = median(perf), error = false, timeout = false)  # Return NamedTuple
			end)
			=>
				[:performance, :error, :timeout],
	)
end

# Expected to be the same:
# `runtime`, `platform`, `analysis`, `input_program`
# May differ:
# `memory_usage`, `completion_time`, `time_unit`, `exception`, `exception_reason`, `timeout`, `timeout_amount`
# Must differ:
# `runtime_iteration`
squash_query = Cols(:runtime, :platform, :analysis, :input_program)
runtime_performance_aggregated = combine(groupby(runtime_performance, squash_query),
	# :performance::Vec{Float64}, :error::Vec{Bool}, :timeout::Vec{Bool}
	[:completion_time, :exception, :exception_reason, :timeout, :timeout_amount] =>
	# combine rows such that:
		(
			(completion_time, exception, exception_reason, timeout, timeout_amount) ->
				if any(Bool.(exception)) # if any row has an error, the combination is considered an error
					(; completion_time = missing, exception = true, exception_reason = first(skipmissing(exception_reason)), timeout = false, timeout_amount = first(skipmissing(timeout_amount)))
				elseif any(Bool.(timeout)) # if none are error, but any row is timeout, the combination is considered to timeout
					(; completion_time = missing, exception = false, exception_reason = missing, timeout = true, timeout_amount = maximum(skipmissing(timeout)))
				else # none should be error or timeout, the combination is considered the median
					(; completion_time = median(completion_time), exception = false, exception_reason = missing, timeout = false, timeout_amount = first(skipmissing(timeout_amount)))
				end
		)
		# the returned named tuple is used as the combination outcome
		=>
			[:completion_time, :exception, :exception_reason, :timeout, :timeout_amount],
)


df_rgular_aggregated = select(filter(row -> row.platform .== "uninstrumented", runtime_performance_aggregated), Not([:platform, :analysis]))
df_wasabi_aggregated = select(filter(row -> row.platform .== "Wasabi", runtime_performance_aggregated), Not(:platform))
df_wastrm_aggregated = select(filter(row -> row.platform .== "Wastrumentation", runtime_performance_aggregated), Not(:platform))
"parsed & squased runtime performance"
#---
if length(unique(df_rgular.runtime_iteration)) > 1
	df_rgular |>
	@vlplot(
		:line,
		encoding = {
			x = {
				field = "runtime_iteration",
				type = "nominal",
			},
			y = {
				aggregate = "median",
				field = "completion_time",
				type = "quantitative",
				scale = {
					type = "log",
				},
				title = "Execution time (ms)",
			},
			color = {
				field = "input_program",
				type = "nominal",
			},
		},
		config = {
			line = {
				point = true,
			},
			scale = {
				useUnaggregatedDomain = true,
			},
		},
	)
else
	"Cannot plot evolution of iterations over a single iteration ..."
end
#---
filter(row -> row.analysis == "forward", df_wasabi) |>
@vlplot(
	:line,
	mark = {
		:errorband,
		extent = :ci,
	},
	encoding = {
		x = {
			field = "runtime_iteration",
			type = "nominal",
			scale = {
				"rangeStep" = 12,
			},
		},
		y = {
			aggregate = "median",
			field = "completion_time",
			type = "quantitative",
			scale = {
				type = "log",
			},
			title = "Execution time (ms)",
		},
		color = {
			field = "input_program",
			type = "nominal",
		},
	},
	config = {
		line = {
			point = true,
		},
		scale = {
			useUnaggregatedDomain = true,
		},
	},
)
#---
filter(row -> row.analysis == "forward", df_wastrm) |>
@vlplot(
	:line,
	mark = {
		:errorband,
		extent = :ci,
	},
	encoding = {
		x = {
			field = "runtime_iteration",
			type = "nominal",
			scale = {
				"rangeStep" = 12,
			},
		},
		y = {
			aggregate = "median",
			field = "completion_time",
			type = "quantitative",
			scale = {
				type = "log",
			},
			title = "Execution time (ms)",
		},
		color = {
			field = "input_program",
			type = "nominal",
		},
	},
	config = {
		line = {
			point = true,
		},
		scale = {
			useUnaggregatedDomain = true,
		},
	},
)
#---
# Plot Runtime Performances
#---
# df_rgular_median_over_iterations = combine(groupby(df_rgular, Cols(:input_program, :runtime, :setup)), :performance => median => :performance)
# df_wasabi_median_over_iterations = combine(groupby(df_wasabi, Cols(:input_program, :runtime, :platform, :analysis)), :performance => median => :performance)
# df_wastrm_median_over_iterations = combine(groupby(df_wastrm, Cols(:input_program, :runtime, :platform, :analysis)), :performance => median => :performance)

# function squash_iterations(df)
#   combine(
#     groupby(df, Cols(:input_program, :runtime, :platform, :analysis)),
#     # :performance::Vec{Float64}, :error::Vec{Bool}, :timeout::Vec{Bool}
#     [:performance, :error, :timeout] =>
#     # combine rows such that:
#     #   if any row has error, the combination is error
#     #   if none are error, but any row is timeout, the combination is timeout
#     #   none should be error or timeout, the combination is the median
#     ((perf, err, to) -> if any(Bool.(err))
#         (; performance = missing, error = true, timeout = false)  # Return NamedTuple
#     elseif any(Bool.(to))
#         (; performance = missing, error = false, timeout = true)  # Return NamedTuple
#     else
#         (; performance = median(perf), error = false, timeout = false)  # Return NamedTuple
#     end)
#     => [:performance, :error, :timeout],
#   )
# end

# df_rgular_median_over_iterations = combine(groupby(df_rgular, Cols(:input_program, :runtime, :setup)), :performance => median => :performance)
# df_wasabi_median_over_iterations = df_wasabi |> squash_iterations
# df_wastrm_median_over_iterations = df_wastrm |> squash_iterations
# "Median over runtimes computed!"
#---
# E.g. overhead wasabi: 10x
#      overhead wastrm: 50x
#
#      ==> wasabi faster (lower overhead; 10 <= 50)
#
#      ==> wasabi / wastrm = 0.2
#      ==> marked as "1. wasabi is much faster"

performance_ordinal_domain = [
	"1. Wastrmnt >3 times slower",
	"2. Wastrmnt 3-1.05 times slower",
	"3. Wastrmnt comparable",
	"4. Wastrmnt 3-1.05 times faster",
	"5. Wastrmnt >3 times faster",
]

#        1
# [-∞ ======= 0.3 ======= 0.95 ======= 1.05 ======= 3 ======= 100 ======= ]
function performance_comparison(n::Float64)
	if n >= 0 && n <= 0.3
		return performance_ordinal_domain[1]
	elseif n > 0.3 && n <= 0.95
		return performance_ordinal_domain[2]
	elseif n > 0.95 && n <= 1.05
		return performance_ordinal_domain[3]
	elseif n > 1.05 && n <= 3
		return performance_ordinal_domain[4]
	elseif n > 3 && n < 100
		return performance_ordinal_domain[5]
	elseif n >= 100
		return "0. ❌ wasabi is INCREADIBLY SLOW"
	else
		return "Input is out of range"
	end
end

function performance_comparison(n::Missing)
	n
end
#---
df_wastrm_joined_rgular = innerjoin(
	rename(df_rgular_aggregated, :completion_time  => :rgular_completion_time,
		:exception        => :rgular_exception,
		:exception_reason => :rgular_exception_reason,
		:timeout          => :rgular_timeout,
		:timeout_amount   => :rgular_timeout_amount),
	rename(df_wastrm_aggregated, :completion_time  => :wastrm_completion_time,
		:exception        => :wastrm_exception,
		:exception_reason => :wastrm_exception_reason,
		:timeout          => :wastrm_timeout,
		:timeout_amount   => :wastrm_timeout_amount),
	on = [:runtime, :input_program],
)
"joined"
#---
df_wasabi_joined_rgular = innerjoin(
	rename(df_rgular_aggregated, :completion_time  => :rgular_completion_time,
		:exception        => :rgular_exception,
		:exception_reason => :rgular_exception_reason,
		:timeout          => :rgular_timeout,
		:timeout_amount   => :rgular_timeout_amount),
	rename(df_wasabi_aggregated, :completion_time  => :wasabi_completion_time,
		:exception        => :wasabi_exception,
		:exception_reason => :wasabi_exception_reason,
		:timeout          => :wasabi_timeout,
		:timeout_amount   => :wasabi_timeout_amount),
	on = [:runtime, :input_program],
)
"joined"
#---
# df_wastrm_instruction_overhead_labeled = transform(
#   outerjoin(
#     select(rename(df_rgular_aggregated, :performance => :rgular_performance), Not([:setup])),
#     select(rename(df_wastrm_aggregated, :performance => :wastrm_performance), Not([:platform])),
#     on=[:runtime, :input_program],
#   ),
#   [:rgular_performance, :wastrm_performance, :error, :timeout] =>
#   ByRow((rgular_performance, wastrm_performance, err, timeout) ->
#   begin
#     @assert !(rgular_performance === missing)
#     @assert !(wastrm_performance === missing && err === missing && timeout === missing)
#     if err
#         [missing, "E"]
#     elseif timeout
#       [missing, "T"]
#     else
#       relative_performance = wastrm_performance / rgular_performance
#       [relative_performance, @sprintf("%.4g", relative_performance)]
#     end
#   end)
#   => [:overhead, :text_report]
# )
#---
# df_wasabi_instruction_overhead = select(
#   rightjoin(rename(select(df_rgular, :input_program, :performance, :runtime_iteration), :performance => :performance_baseline), df_wasabi, on=[:runtime_iteration, :input_program]),
#   [:performance, :performance_baseline]
#     => ((performance, performance_baseline) -> performance ./ performance_baseline)
#     => :overhead,
#   # What to keep:
#   :input_program, :runtime_iteration, :runtime, :platform, :analysis
# )

# df_wastrm_instruction_overhead = select(
#   rightjoin(rename(select(df_rgular, :input_program, :performance, :runtime_iteration), :performance => :performance_baseline), df_wastrm, on=[:runtime_iteration, :input_program]),
#   [:performance, :performance_baseline]
#     => ((performance, performance_baseline) -> performance ./ performance_baseline)
#     => :overhead,
#   # What to keep:
#   :input_program, :runtime_iteration, :runtime, :platform, :analysis
# )

# "Overheads computed!"
#---
# # Aggregate all overhead
# df_wasabi_instruction_overhead_single = combine(groupby(df_wasabi_instruction_overhead, Cols(:input_program, :runtime, :platform, :analysis)), :overhead => median => :overhead)
# df_wastrm_instruction_overhead_single = combine(groupby(df_wastrm_instruction_overhead, Cols(:input_program, :runtime, :platform, :analysis)), :overhead => median => :overhead)
# "Aggregated!"
#---
df_wasabi_instruction_overhead_single = transform(
	df_wasabi_joined_rgular,
	[:rgular_completion_time, :wasabi_completion_time, :wasabi_exception, :wasabi_exception_reason, :wasabi_timeout]
	=>
		ByRow((rgular_completion_time, wasabi_completion_time, wasabi_exception, wasabi_exception_reason, wasabi_timeout) -> begin
			@assert !(rgular_completion_time === missing)
			if wasabi_timeout
				return ("Wasabi", missing, "T")
			elseif wasabi_exception
				return ("Wasabi", missing, "MD")
			else
				return ("Wasabi", wasabi_completion_time / rgular_completion_time, wasabi_completion_time / rgular_completion_time)
			end
		end)
		=>
			[:platform, :overhead, :text_report],
)
"wasabi_overhead_plot computed!"
#---
df_wastrm_instruction_overhead_single = transform(
	df_wastrm_joined_rgular,
	[:rgular_completion_time, :wastrm_completion_time, :wastrm_exception, :wastrm_exception_reason, :wastrm_timeout]
	=>
		ByRow(
			(rgular_completion_time, wastrm_completion_time, wastrm_exception, wastrm_exception_reason, wastrm_timeout) -> begin
				@assert !(rgular_completion_time === missing)
				if wastrm_timeout
					return ("Wastrumentation", missing, "T")
				elseif wastrm_exception
					return ("Wastrumentation", missing, "MD")
				else
					return ("Wastrumentation", wastrm_completion_time / rgular_completion_time, wastrm_completion_time / rgular_completion_time)
				end
			end,
		)
		=>
			[:platform, :overhead, :text_report],
)
"wastrm_overhead_plot computed!"
#---
wasabi_overhead_plot =
	df_wasabi_instruction_overhead_single |>
	@vlplot(
		"transform" = [
			{
				"calculate" = "substring(datum.text_report, 0, 5)",
				"as" = "overhead_truncated",
			},
		],
		encoding = {
			y = {
				field = "analysis",
				type = "nominal",
			},
			x = {
				field = "input_program",
				type = "nominal",
				axis = {title = "Input Program"},
			},
		},
		layer = [
			{
				mark = "rect",
				encoding = {
					color = {
						field = "overhead",
						type = "quantitative",
						scale = {
							type = "log",
							scheme = "blues",
						},
						legend = {
							title = "Overhead for Wasabi",
							orient = "top",
							titleLimit = 1000,
							gradientLength = 400,
							titleAnchor = "center",
							titleAlign = "center",
						},
					},
				},
			},
			{
				mark = {
					type = "text",
					fontSize = "6",
				},
				encoding = {
					text = {
						field = "overhead_truncated",
						type = "nominal",
					},
				},
			},
		],
		config = {
			axis = {grid = true, tickBand = "extent"},
		},
	)

wasabi_overhead_plot |> save(target_dir_prefix * "wasabi-overhead.pdf")
wasabi_overhead_plot
#---
wastrm_overhead_plot_single =
	df_wastrm_instruction_overhead_single |>
	@vlplot(
		"transform" = [
			{
				"calculate" = "substring(datum.text_report, 0, 5)",
				"as" = "text_report_truncated",
			},
		],
		encoding = {
			y = {
				field = "analysis",
				type = "nominal",
				axis = {labelAngle = "-30"},
			},
			x = {
				field = "input_program",
				type = "nominal",
				axis = {labelAngle = "-30", title = "Input Program"},
			},
		},
		layer = [
			{
				mark = "rect",
				encoding = {
					color = {
						field = "overhead",
						type = "quantitative",
						scale = {
							type = "log",
							scheme = "blues",
						},
						legend = {
							title = "Overhead for Wastrumentation",
							orient = "top",
							titleLimit = 1000,
							gradientLength = 540,
							titleAnchor = "center",
							titleAlign = "center",
						},
					},
				},
			},
			{
				mark = {
					type = "text",
					fontSize = "6",
				},
				encoding = {
					text = {
						field = "text_report_truncated",
						type = "nominal",
					},
				},
			},
		],
		config = {
			axis = {grid = true, tickBand = "extent"},
		},
	)


wastrm_overhead_plot_single |> save(target_dir_prefix * "wastrumentation-runtime-overhead-plot-single.pdf")
wastrm_overhead_plot_single
#---
wastrm_overhead_plot =
	df_wastrm_instruction_overhead_single |>
	@vlplot(
		"transform" = [
			{
				"calculate" = "substring(datum.text_report, 0, 5)",
				"as" = "text_report_truncated",
			},
		],
		"spacing" = 15,
		"bounds" = "flush",
		"vconcat" = [
			{
				"mark" = {
					"type" = "boxplot",
					"extent" = "min-max",
				},
				"height" = 100,
				"encoding" = {
					"x" = {
						field = "input_program",
						type = "nominal",
						"axis" = false,
					},
					"y" = {
						field = "overhead",
						type = "quantitative",
						"scale" = {type = "log", domainMin = 1},
						"title" = "",
					},
				},
			},
			{
				"spacing" = 15,
				"bounds" = "flush",
				"hconcat" = [
					{
						encoding = {
							y = {
								field = "analysis",
								type = "nominal",
								axis = {labelAngle = "-30"},
							},
							x = {
								field = "input_program",
								type = "nominal",
								axis = {title = "Input Program", labelAngle = "-30"},
							},
						},
						layer = [
							{
								mark = "rect",
								encoding = {
									color = {
										field = "overhead",
										type = "quantitative",
										scale = {
											type = "log",
											scheme = "blues",
										},
										legend = {
											title = "Overhead for Wastrumentation",
											orient = "top",
											titleLimit = 1000,
											gradientLength = 540,
											titleAnchor = "center",
											titleAlign = "center",
										},
									},
								},
							},
							{
								mark = {
									type = "text",
									fontSize = "6",
								},
								encoding = {
									text = {
										field = "text_report_truncated",
										type = "nominal",
									},
								},
							},
						],
					},
					{
						"mark" = {
							"type" = "boxplot",
							"extent" = "min-max",
						},
						"width" = 100,
						"encoding" = {
							"y" = {
								field = "analysis",
								type = "nominal",
								"axis" = false,
							},
							"x" = {
								field = "overhead",
								type = "quantitative",
								"scale" = {type = "log", domainMin = 1},
								"title" = "",
							},
						},
					},
				],
			},
		],
		config = {
			axis = {grid = true, tickBand = "extent"},
		},
	)

wastrm_overhead_plot |> save(target_dir_prefix * "wastrumentation-runtime-overhead-plot.pdf")
wastrm_overhead_plot
#---
all_performance_overhead = vcat(
	select(df_wastrm_instruction_overhead_single, Not([:wastrm_completion_time, :wastrm_exception, :wastrm_exception_reason, :wastrm_timeout, :wastrm_timeout_amount])),
	select(df_wasabi_instruction_overhead_single, Not([:wasabi_completion_time, :wasabi_exception, :wasabi_exception_reason, :wasabi_timeout, :wasabi_timeout_amount])),
)

all_performance_overhead_plot =
	all_performance_overhead |> @vlplot(
		"transform" = [
			{
				"calculate" = "substring(datum.text_report, 0, 5)",
				"as" = "text_report_truncated",
			},
		],
		facet = {
			row = {
				field = "platform",
				type = "nominal",
			},
		},
		spec = {
			layer = [
				{
					mark = "rect",
					encoding = {
						y = {
							field = "analysis",
							type = "nominal",
							axis = {labelAngle = "-30"},
						},
						x = {
							field = "input_program",
							type = "nominal",
							axis = {title = "Input Program"},
							axis = {labelAngle = "-30"},
						},
						color = {
							field = "overhead",
							type = "quantitative",
							scale = {
								type = "log",
								scheme = "blues",
							},
							legend = {
								title = "Runtime Overhead (X)",
								orient = "top",
								titleLimit = 1000,
								gradientLength = 540,
								titleAnchor = "center",
								titleAlign = "center",
							},
						},
					},},
				{
					mark = {
						type = "text",
						fontSize = "6",
					},
					encoding = {
						y = {
							field = "analysis",
							type = "nominal",
						},
						x = {
							field = "input_program",
							type = "nominal",
							axis = {title = "Input Program"},
						},
						text = {
							field = "text_report_truncated",
							type = "nominal",
						},
					},
				},
			],},
		config = {
			axis = {
				grid = true,
				tickBand = "extent",
			},
		},
	)


all_performance_overhead_plot |> save(target_dir_prefix * "wastrumentation-wasabi-runtime-overhead-single.pdf")
all_performance_overhead_plot
#---
df_wasabi_wastrm_joined = innerjoin(
	rename(df_wastrm_aggregated, :completion_time  => :wastrm_completion_time,
		:exception        => :wastrm_exception,
		:exception_reason => :wastrm_exception_reason,
		:timeout          => :wastrm_timeout,
		:timeout_amount   => :wastrm_timeout_amount),
	rename(df_wasabi_aggregated, :completion_time  => :wasabi_completion_time,
		:exception        => :wasabi_exception,
		:exception_reason => :wasabi_exception_reason,
		:timeout          => :wasabi_timeout,
		:timeout_amount   => :wasabi_timeout_amount),
	on = [:runtime, :analysis, :input_program],
)

df_wasabi_wastrm_runtime_relative = transform(
	df_wasabi_wastrm_joined,
	[:wastrm_completion_time, :wastrm_exception, :wastrm_exception_reason, :wastrm_timeout, :wastrm_timeout_amount, :wasabi_completion_time, :wasabi_exception, :wasabi_exception_reason, :wasabi_timeout, :wasabi_timeout_amount] =>
		ByRow(
			(wastrm_completion_time, wastrm_exception, wastrm_exception_reason, wastrm_timeout, wastrm_timeout_amount, wasabi_completion_time, wasabi_exception, wasabi_exception_reason, wasabi_timeout, wasabi_timeout_amount) ->
				begin
					if wastrm_exception || wastrm_timeout || wasabi_exception || wasabi_timeout
						if wastrm_exception && wasabi_exception
							[missing, "MD:M&S"]
						elseif wastrm_timeout && wasabi_timeout
							[missing, "T:M&S"]
						elseif wastrm_exception && wasabi_timeout
							[missing, "MD:M,T:S"]
						elseif wastrm_timeout && wasabi_exception
							[missing, "T:M,MD:S"]
						elseif wastrm_exception
							[missing, "MD:M"]
						elseif wasabi_exception
							[missing, "MD:S"]
						elseif wastrm_timeout
							[missing, "T:M"]
						elseif wasabi_timeout
							[missing, "T:S"]
						end
					else
						relative_performance = wasabi_completion_time / wastrm_completion_time
						[relative_performance, @sprintf("%.4g", relative_performance)]
					end
				end,
		)
		=>
			[:wasabi_perf_per_wastrm_perf, :text_report],
)

"computed"
#---
df_wasabi_wastrm_runtime_relative_plot =
	df_wasabi_wastrm_runtime_relative |> @vlplot(
		"transform" = [
			{
				"calculate" = "substring(datum.text_report, 0, 5)",
				"as" = "text_report_truncated",
			},
		],
		encoding = {
			y = {
				field = "analysis",
				type = "nominal",
				axis = {labelAngle = "-30"},
			},
			x = {
				field = "input_program",
				type = "nominal",
				axis = {title = "Input Program", labelAngle = "-30"},
			},
		},
		layer = [
			{
				mark = "rect",
				encoding = {
					color = {
						field = "wasabi_perf_per_wastrm_perf",
						type = "quantitative",
						scale = {
							type = "log",
							domainMid = 1,
							scheme = "redyellowgreen",
						},
						legend = {
							title = "Wasabi Execution Time / Wastrumentation Execution Time",
							orient = "top",
							titleLimit = 1000,
							gradientLength = 400,
							titleAnchor = "center",
							titleAlign = "center",
						},
					},
				},
			},
			{
				mark = {
					type = "text",
					fontSize = "6",
				},
				encoding = {
					text = {
						field = "text_report_truncated",
						type = "nominal",
					},
				},
			},
		],
		config = {
			axis = {grid = true, tickBand = "extent"},
		},
	)

df_wasabi_wastrm_runtime_relative_plot |> save(target_dir_prefix * "df-wasabi-wastrm-runtime-relative.pdf")
df_wasabi_wastrm_runtime_relative_plot
#---

MD_MS    = nrow(filter(:text_report => (tr) -> tr .=== "MD:M&S", df_wasabi_wastrm_runtime_relative))
T_MS     = nrow(filter(:text_report => (tr) -> tr .=== "T:M&S", df_wasabi_wastrm_runtime_relative))
MD_M_T_S = nrow(filter(:text_report => (tr) -> tr .=== "MD:M,T:S", df_wasabi_wastrm_runtime_relative))
T_M_MD_S = nrow(filter(:text_report => (tr) -> tr .=== "T:M,MD:S", df_wasabi_wastrm_runtime_relative))
MD_M     = nrow(filter(:text_report => (tr) -> tr .=== "MD:M", df_wasabi_wastrm_runtime_relative))
MD_S     = nrow(filter(:text_report => (tr) -> tr .=== "MD:S", df_wasabi_wastrm_runtime_relative))
T_M      = nrow(filter(:text_report => (tr) -> tr .=== "T:M", df_wasabi_wastrm_runtime_relative))
T_S      = nrow(filter(:text_report => (tr) -> tr .=== "T:S", df_wasabi_wastrm_runtime_relative))

passed = nrow(filter(:wasabi_perf_per_wastrm_perf => (tr) -> tr .!== missing, df_wasabi_wastrm_runtime_relative))
total  = nrow(df_wasabi_wastrm_runtime_relative)

@assert MD_MS == 0
# @assert T_MS     == 0 => not null
@assert MD_M_T_S == 0
@assert T_M_MD_S == 0
@assert MD_M == 0
@assert MD_S == 0
# @assert T_M      == 0 => not null
# @assert T_S      == 0 => not null

sentence = "
We summarize the labels and occurences.
For a total of $(total) combinations where both Wastrumentation and Wasabi yield a valid instrumented module, $(passed) combinations yield a measurement.
For $(T_MS) combinations both Wastrumentation and Wasabi timeout.
For $(T_M) combinations only Wastrumentation reports a timeout.
For $(T_S) combinations only Wastrumentation reports a timeout.
"

println(sentence)
#---
# Let's do the same performance evaluation, but now take the first run!
#---
# baseline =
#     rename(
#       select(
#         subset(df_rgular, :runtime_iteration => i -> i .== 1),
#         Not([:setup]),
#       ),
#       :performance => :performance_baseline,
#     )

# df_wasabi_timeout_computed = df_wasabi
# df_wastrm_timeout_computed = df_wastrm

# if isa(df_wasabi.performance, Vector{String})
#   df_wasabi_timeout_computed = transform(df_wasabi, :performance => ByRow((x) -> parse(Float64, x == "timeout 10s" ? "10000" : x)) => :performance)
# end
# if isa(df_wastrm.performance, Vector{String})
#   df_wastrm_timeout_computed = transform(df_wastrm, :performance => ByRow((x) -> parse(Float64, x == "timeout 10s" ? "10000" : x)) => :performance)
# end

# @assert isa(df_wasabi_timeout_computed.performance, Vector{Float64}) "df_wasabi should be parsed to Float64"
# @assert isa(df_wastrm_timeout_computed.performance, Vector{Float64}) "df_wasabi should be parsed to Float64"

# using Statistics

# # Aggregate computations per 'run'!
# df_wasabi_aggr = combine(groupby(df_wasabi_timeout_computed, Cols(:runtime_iteration, :setup, :runtime, :input_program, "time-unit")), :performance => median => :performance)
# df_wastrm_aggr = combine(groupby(df_wastrm_timeout_computed, Cols(:runtime_iteration, :setup, :runtime, :input_program, "time-unit")), :performance => median => :performance)

# df_wasabi_sep_analyses = transform(
#   df_wasabi_aggr,
#   :setup => ByRow(setup -> match(r"\[wasabi - (.+)\]", setup).captures[1]) => :analysis,
#   :setup => (_ -> "wasabi") => :setup,
# )
# df_wastrm_sep_analyses = transform(
#   df_wastrm_aggr,
#   :setup => ByRow(setup -> match(r"\[wastrumentation - (.+)\]", setup).captures[1]) => :analysis,
#   :setup => (_ -> "wastrumentation") => :setup,
# )

# df_wasabi_instruction_overhead = select(
#   innerjoin(baseline, df_wasabi_sep_analyses, on=[:input_program, :runtime, :runtime_iteration, "time-unit"]),
#   [:performance, :performance_baseline]
#     => ((performance, performance_baseline) -> performance ./ performance_baseline)
#     => :overhead,
#   # What to keep:
#   :input_program, :setup, :analysis,
# )

# df_wastrm_instruction_overhead = select(
#   innerjoin(baseline, df_wastrm_sep_analyses, on=[:input_program, :runtime, :runtime_iteration, "time-unit"]),
#   [:performance, :performance_baseline]
#     => ((performance, performance_baseline) -> performance ./ performance_baseline)
#     => :overhead,
#   # What to keep:
#   :input_program, :setup, :analysis,
# )

# # Aggregate all overhead
# df_wasabi_instruction_overhead_single = combine(groupby(df_wasabi_instruction_overhead, Cols(:input_program, :setup, :analysis)), :overhead => median => :overhead)
# df_wastrm_instruction_overhead_single = combine(groupby(df_wastrm_instruction_overhead, Cols(:input_program, :setup, :analysis)), :overhead => median => :overhead)

# df_wasabi_wastrm_overhead = transform(
#   innerjoin(
#     rename(select(df_wasabi_instruction_overhead_single, Not(:setup)), :overhead => :overhead_wasabi),
#     rename(select(df_wastrm_instruction_overhead_single, Not(:setup)), :overhead => :overhead_wastrm),
#     on=[:input_program, :analysis]
#   ),
#   [:overhead_wasabi, :overhead_wastrm]
#     => ((overhead_wasabi, overhead_wastrm) -> overhead_wasabi ./ overhead_wastrm)
#     => :time_for_wasabi_per_time_for_wastrm,
# )

# df_wasabi_wastrm_overhead = transform(
#   df_wasabi_wastrm_overhead,
#   :time_for_wasabi_per_time_for_wastrm
#   =>
#   ByRow(performance_comparison)
#   =>
#   :time_for_wasabi_per_time_for_wastrm,
# ) |>
# @vlplot(
#   :rect,
#   encoding={
#     color={
#       field="time_for_wasabi_per_time_for_wastrm",
#       type="ordinal",
#       scale={
#         scheme="blueorange",
#         domain=performance_ordinal_domain
#       },
#     },
#     x={
#       field="analysis",
#       type="nominal",
#     },
#     y={
#       field="input_program",
#       type="nominal",
#     },
#   },
#   config={
#     spacing=100,
#     view={stroke=:transparent},
#     axis={domainWidth=1}
#   },
# )
#---
# exit()
#---
using CSV, DataFrames, Dates
#---
df_memoization_performance = DataFrame(load(target_dir_prefix * "memoization_performance_wasmr3.csv"))
#---
function parse_time_to_ms(time::String)
	if time === "NA"
		missing
	elseif occursin("ms", time)
		time_element = match(r"(\d+\.?\d*)ms", time).captures[1]
		parse(Float64, time_element)
	elseif occursin("s", time)
		time_element = match(r"(\d+.?\d*)s", time).captures[1]
		parse(Float64, time_element) * 1000
	else
		error("Could not parse $time")
	end
end

# Apply the parsing function to the columns
df_memoization_performance.uninstrumented = parse_time_to_ms.(df_memoization_performance.uninstrumented)
df_memoization_performance.instrumented = parse_time_to_ms.(df_memoization_performance.instrumented)

"Interpretation done"
#---
df_memoization_performance_overhead = transform(
	df_memoization_performance,
	[:instrumented, :uninstrumented] => ByRow((instrumented, uninstrumented) ->
		if instrumented !== missing && uninstrumented !== missing
			instrumented / uninstrumented
		else
			missing
		end
	) => :overhead,
)

df_non_missing_memoization_performance_overhead = filter(row -> row.overhead !== missing, df_memoization_performance_overhead)

df_memoization_performance_overhead_slower = filter(row -> row.overhead > 1, df_non_missing_memoization_performance_overhead)
df_memoization_performance_overhead_faster = filter(row -> row.overhead < 1, df_non_missing_memoization_performance_overhead)
df_memoization_performance_overhead_faster = transform(
	df_memoization_performance_overhead_faster,
	[:instrumented, :uninstrumented] => ByRow((instrumented, uninstrumented) -> uninstrumented / instrumented) => :overhead,
)
"Interpreted all speedups and slowdowns"
#---
least_slower = df_memoization_performance_overhead_slower[argmin(df_memoization_performance_overhead_slower.overhead), :]
hghst_slower = df_memoization_performance_overhead_slower[argmax(df_memoization_performance_overhead_slower.overhead), :]
least_slower_prog = least_slower.input_program
least_slower_vlue = least_slower.overhead
hghst_slower_prog = hghst_slower.input_program
hghst_slower_vlue = hghst_slower.overhead

least_faster = df_memoization_performance_overhead_faster[argmin(df_memoization_performance_overhead_faster.overhead), :]
hghst_faster = df_memoization_performance_overhead_faster[argmax(df_memoization_performance_overhead_faster.overhead), :]
least_faster_prog = least_faster.input_program
least_faster_vlue = least_faster.overhead
hghst_faster_prog = hghst_faster.input_program
hghst_faster_vlue = hghst_faster.overhead

sentence = "
Our benchmarks on the memoization analysis show that the analysis can incur a performance penalty but can also speed up the input program execution.
The performance penalty ranges from $(@sprintf("%.2f", least_slower_vlue))x slowdown ($least_slower_prog) to $(@sprintf("%.2f", hghst_slower_vlue))x slowdown ($hghst_slower_prog).
The speed up ranges from $(@sprintf("%.2f", least_faster_vlue))x speedup ($least_faster_prog) to $(@sprintf("%.2f", hghst_faster_vlue))x speedup ($hghst_faster_prog).
"

println(sentence)
#---
# Split the columns for 'uninstrumented' and 'instrumented' and make them two DF's that we then append to each other
df_memoization_uninstrumented = select(df_memoization_performance, :input_program, :uninstrumented)
df_memoization_instrumented = select(df_memoization_performance, :input_program, :instrumented)

rename!(df_memoization_uninstrumented, :uninstrumented => :runtime)
rename!(df_memoization_instrumented, :instrumented => :runtime)

df_memoization_uninstrumented[!, :type] .= "uninstrumented"
df_memoization_instrumented[!, :type] .= "instrumented"

df_memoization_combined = vcat(df_memoization_uninstrumented, df_memoization_instrumented)
"Merged results into single view"
#---
df_memoization_combined_type_renamed = transform(df_memoization_combined, :type => ByRow((type) -> begin
	if type .=== "uninstrumented"
		"Regular Execution"
	elseif type .=== "instrumented"
		"Memoized Execution"
	else
		error("Unknown case in $type")
	end
end) => :type)
"Renamed"
#---
df_memoization_combined_plot =
	df_memoization_combined_type_renamed |> @vlplot(
		mark = {
			type = "bar",
			cornerRadiusEnd = 3,  # Adds rounded corners at the top of each bar
		},
		encoding = {
			color = {
				field = "type",
				type = "nominal",
				scale = {
					scheme = "pastel2",  # Color scheme for distinct colors that are accessible
					# range=["instrumented", "uninstrumented"],  # Original values in `type`
					# domain=["Memoized Execution", "Regular Execution"],  # Custom legend names
				},
				legend = {
					title = "Execution model",
					orient = "top",
					titleFontSize = 12,
					labelFontSize = 10,
				},
			},
			xOffset = {
				field = "type",
				type = "nominal",
			},
			x = {
				field = "input_program",
				type = "nominal",
				axis = {
					title = "Input Program",
					labelAngle = -30,
					labelFontSize = 10,
					titleFontSize = 12,
					titlePadding = 10,
				},
			},
			y = {
				field = "runtime",
				type = "quantitative",
				axis = {
					title = "Runtime (ms)",
					titleFontSize = 12,
					labelFontSize = 10,
					grid = false # Removes y-axis gridlines for a cleaner look
				},
				scale = {
					type = "log",
				},
			},
		},
		title = "Runtime Comparison for Regular Execution and Memoized Execution",  # Adds a descriptive title
		config = {
			axis = {
				domainColor = "#999",  # Lightens axis lines for a cleaner look
			},
			view = {stroke = :transparent},
		}
	)

df_memoization_combined_plot |> save(target_dir_prefix * "memoization-over-head-plot.pdf")
df_memoization_combined_plot
