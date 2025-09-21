ConvertFrom-Json | Select-Object ticker, @{Name="得分"; Expression="score"}, @{Name="單月漲幅"; Expression="one_month_price_change"}, @{Name="長期回報"; Expression="long_term_total_return"} | Format-Table
