# Rust Pipe

# Usage

```
rp (rust pipe) - 0.3.0 - 2026-01-19 02:34:33

A command-line string processing tool implemented in Rust that supports streaming processing.

Usage: rp [<options> [<option_value>]] [<input_cmd>] [<op_cmd>[ ...]] [<output_cmd>]

<options> 选项：
 -V,--version    打印版本信息。
 -h,--help       打印帮助信息。
                 -h|-help[ <topic>]
                     <topic>     帮助主题：
                         opt|options     打印选项帮助信息。
                         in|input        打印数据输入命令帮助信息。
                         op              打印数据操作命令帮助信息。
                         out|output      打印数据输出命令帮助信息。
                         code            打印退出码帮助信息。
                         fmt             打印格式化帮助信息。
                         cond|condition  打印条件表达式帮助信息。
                                 未指定则打印全部帮助信息。
 -v,--verbose    执行之前打印流水线详情。
 -d,--dry-run    仅解析流水线，不执行。
 -n,--nocase     全局忽略大小写。
 -s,--skip-err 全局忽略错误。
 -t,--token      以Token模式解析下一个参数。
                 除了紧跟的第一个参数外，其他参数会被忽略。
                 -t|--token <token>
                     <token> 需要解析的文本参数，必选。
                 例如：
                     -e ':in :uniq :to out'

<input_cmd> 数据输入命令：
 :in         从标准输入读取输入。
             未指定元素输入时的默认输入。
 :file       从文件读取输入。
             :file <file>[ <file>][...]
                 <file>  文件路径，至少指定一个。
             例如：
                 :file input.txt
                 :file input1.txt input2.txt input3.txt
 :clip       从剪切板读取输入。
 :of         使用直接字面值作为输入。
             :of <text>[ <text][...]
                 <text>  字面值，至少指定一个，如果以':'开头，需要使用'\:'转义。
             例如：
                 :of line
                 :of line1 "line 2" 'line 3'
 :gen        生成指定范围内的整数作为输入，支持进一步格式化。
             :gen <start>[,[<end>][,<step>]][ <fmt>]
                 <start> 起始值，包含，必须。
                 <end>   结束值，包含，可选。
                         未指定时生成到整数最大值（取决于构建版本）。
                         如果范围为空（起始值大于结束值），则无数据生成。
                 <step>  步长，不能为0，可选，未指定时取步长为1。
                         如果步长为正值，表示正序生成；
                         如果步长为负值，表示逆序生成。
                 <fmt>   格式化字符串，以{v}表示生成的整数值。
                         更多格式化信息参考`-h fmt`。
             例如：
                 :gen 0          生成：0 1 2 3 4 5 ...
                 :gen 0,         生成：0 1 2 3 4 5 ...
                 :gen 0,10       生成：0 1 2 3 4 5 6 7 8 9
                 :gen 0,10,2     生成：0 2 4 6 8
                 :gen 0,,2       生成：0 2 4 6 8 10 12 14 ...
                 :gen 10,0       无数据生成
                 :gen 0,10,-1    生成：9 8 7 6 5 4 3 2 1
                 :gen 0,10 n{v}  生成：n0 n1 n2 n3 n4 n5 n6 n7 n8 n9
                 :gen 0,10 "Hex of {v} is {v:#04x}" 生成：
                                 "Hex of 0 is 0x00"
                                 "Hex of 1 is 0x01"
                                 ...
 :repeat     重复字面值作为输入。
             :repeat <value>[ <count>]
                 <value> 需要重复的字面值，必选。
                 <count> 需要重复的次数，必须为非负数，可选，未指定时重复无限次数。

<op_cmd> 数据操作命令：
 :peek       打印每个值到标准输出或文件。
             :peek[ <file>][ append][ lf|crlf]
                 <file>  文件路径，可选。
                 append  追加输出而不是覆盖，可选，如果未指定则覆盖源文件。
                 lf|crlf 指定换行符为'LF'或'CRLF'，可选，如果未指定则默认使用'LF'。
             例如：
                 :peek
                 :peek file.txt
                 :peek file.txt append
                 :peek file.txt lf
                 :peek file.txt crlf
                 :peek file.txt append crlf
 :upper      转为ASCII大写。
 :lower      转为ASCII小写。
 :case       切换ASCII大小写。
 :replace    替换字符串。
             :replace <from> <to>[ <count>][ nocase]
                 <from>  待替换的字符串，必选。
                 <to>    待替换为的字符串，必选。
                 <count> 对每个元素需要替换的次数，必须为正整数，可选，未指定则替换所有。
                 nocase  替换时忽略大小写，可选，未指定时不忽略大小写。
             例如：
                 :replace abc xyz
                 :replace abc xyz 10
                 :replace abc xyz nocase
                 :replace abc xyz 10 nocase
 :trim       去除首尾指定的子串。
             :trim[ <pattern>[ nocase]]
                 <pattern>   需要去除的子串，可选，留空则去除空白字符。
                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
 :ltrim      去除首部指定的子串。
             :ltrim[ <pattern>[ nocase]]
                 <pattern>   需要去除的子串，可选，留空则去除空白字符。
                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
 :rtrim      去除尾部指定的子串。
             :rtrim[ <pattern>[ nocase]]
                 <pattern>   需要去除的子串，可选，留空则去除空白字符。
                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
 :trimc      去除首尾指定范围内的字符。
             :trimc[ <pattern>[ nocase]]
                 <pattern>   需要去除的字符，可选，留空则去除空白字符。
                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
 :ltrimc     去除首部指定范围内的字符。
             :ltrimc[ <pattern>[ nocase]]
                 <pattern>   需要去除的字符，可选，留空则去除空白字符。
                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
 :rtrimc     去除尾部指定范围内的字符。
             :rtrimc[ <pattern>[ nocase]]
                 <pattern>   需要去除的字符，可选，留空则去除空白字符。
                 nocase      忽略大小写，可选，仅当指定了<pattern>时生效。
 :uniq       去重。
             :uniq[ nocase]
                 nocase  去重时忽略大小写，可选，未指定时不忽略大小写。
             例如：
                 :uniq
                 :uniq nocase
 :join       合并数据。
             :join<[ <delimiter>[ <prefix>[ <postfix>[ <batch>]]]]
                 <delimiter> 分隔字符串，可选。
                 <prefix>    前缀字符串，可选。
                             指定前缀字符串时必须指定分割字符串。
                 <postfix>   后缀字符串，可选。
                             指定后缀字符串时必须指定分割字符串和前缀字符串。
                 <batch>     分组大小，必须为正整数，可选，未指定时所有数据为一组。
                             指定分组大小时必须指定分隔字符串、前缀字符串和后缀字符串。
             例如：
                 :join ,
                 :join , [ ]
                 :join , [ ] 3
 :drop       根据指定条件选择数据丢弃，其他数据保留。
             :drop <condition>
                 <condition> 条件表达式，参考`-h cond`或`-h condition`
 :take       根据指定条件选择数据保留，其他数据丢弃。
             :take <condition>
                 <condition> 条件表达式，参考`-h cond`或`-h condition`
 :drop while 根据指定条件选择数据持续丢弃，直到条件首次不满足。
             :drop while <condition>
                 <condition> 条件表达式，参考`-h cond`或`-h condition`
 :take while 根据指定条件选择数据持续保留，直到条件首次不满足。
             :take while <condition>
                 <condition> 条件表达式，参考`-h cond`或`-h condition`
 :count      统计数据数量。
             :count
 :sort       排序。
             :sort[ num [<default>]][ nocase][ desc][ random]
                 num         按照数值排序，可选，未指定时按照字典序排序。
                             尝试将文本解析为数值后排序，无法解析的按照<default>排序。
                 <default>   仅按照数值排序时生效，无法解析为数值的文本的默认数值，可选，
                             未指定时按照数值最大值处理。
                 nocase      忽略大小写，仅按字典序排序时生效，可选，未指定时不忽略大小写。
                 desc        逆序排序，可选，未指定时正序排序。
                 random      随机排序，与按照数值排序和字典序排序互斥，且不支持逆序。
             例如：
                 :sort
                 :sort desc
                 :sort nocase
                 :sort nocase desc
                 :sort num
                 :sort num desc
                 :sort num 10
                 :sort num 10 desc
                 :sort num 10.5
                 :sort num 10.5 desc
                 :sort random

<output_cmd> 数据输出命令：
 :to out     输出到标准输出。
             未指定元素输出时的默认输出。
 :to file    输出到文件。
             :to file <file>[ append][ lf|crlf]
                 <file>  文件路径，必选。
                 append  追加输出而不是覆盖，可选，如果未指定则覆盖源文件。
                 lf|crlf 指定换行符为'LF'或'CRLF'，可选，如果未指定则默认使用'LF'。
             例如：
                 :to file out.txt
                 :to file out.txt append
                 :to file out.txt crlf
                 :to file out.txt lf
                 :to file out.txt append crlf
                 :to file out.txt append lf
 :to clip    输出到剪切板。
             :to clip[ lf|crlf]
                 lf|crlf 指定换行符为'LF'或'CRLF'，可选，如果未指定则默认使用'LF'。
             例如：
                 :to clip
                 :to clip lf
                 :to clip crlf

格式化：（TODO）

条件表达式：
 len [!][<min>],[<max>]
     按照字符串长度范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
     例如：
         len 2,
         len 2,5
         len ,5
         len !,5
         len !2,5
 len [!]=<len>
     按照字符串特定长度选择，支持可选的否定。
     例如：
         len =3
         len !=3
 num [!][<min>],[<max>]
     按照数值范围选择，范围表达式最小值和最大值至少指定其一，支持可选的否定。
     如果无法解析为数则不选择。
     例如：
         num 2,5
         num -2.1,5
         num 2,5.3
         num ,5.3
         num !1,5.3
 num [!]=<spec>
     按照数值特定值选择，支持可选的否定。
     如果无法解析为数则不选择。
     例如：
         num =3
         num =3.3
         num !=3.3
 num[ [!][integer|float]]
     按照整数或浮点数选择，如果不指定则选择数值数据，支持可选的否定。
     例如：
         num
         num integer
         num float
         num !
         num !integer
         num !float
 upper|lower
     选择全部为大写或小写字符的数据，不支持大小写的字符总是满足。
 empty|blank
     选择没有任何字符或全部为空白字符的数据。
 reg <exp>
     选择匹配给定正则表达式的数据。
     <exp>   正则表达式，必选。
     例如：
         reg '\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}'

命令退出码：
 1       解析配置Token失败。
 2       解析输入Token失败。
 3       解析操作Token失败。
 4       解析输出Token失败。
 5       参数解析失败。
 6       命令缺少参数。
 7       参数内容无法完全解析，存在剩余无法解析的内容。
 8       未知参数。
 9       从剪切板读取数据失败。
 10      从文件读取数据失败。
 11      写入数据到剪切板失败。
 12      打开文件失败。
 13      写入数据到文件失败。
 14      格式化字符串失败。
 15      解析正则表达式失败。
 16      解析数值失败。
 17      无效的转义。
```