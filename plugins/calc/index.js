/**
 * Calc 计算器插件
 *
 * 功能：
 * 1. 数学表达式求值（安全沙箱）
 * 2. 单位换算（长度/重量/温度/存储）
 * 3. 百分比计算
 * 4. 常用公式参考
 */

// ==================== 安全计算引擎 ====================

/**
 * 安全的数学表达式求值
 * 仅允许数字、运算符、括号和数学函数
 */
function safeEval(expr) {
  // 清理输入
  expr = expr.replace(/\s/g, '');

  // 白名单校验：只允许数字、运算符、括号、点号和数学函数
  var sanitized = expr.replace(/[0-9+\-*/().%^MathPIeEsincoatanlqrtbscelifloorroundabss]/g, '');
  if (sanitized.length > 0) {
    return { error: '包含非法字符: ' + sanitized };
  }

  try {
    // 替换常见数学函数
    expr = expr.replace(/sin\(/g, 'Math.sin(');
    expr = expr.replace(/cos\(/g, 'Math.cos(');
    expr = expr.replace(/tan\(/g, 'Math.tan(');
    expr = expr.replace(/sqrt\(/g, 'Math.sqrt(');
    expr = expr.replace(/abs\(/g, 'Math.abs(');
    expr = expr.replace(/ceil\(/g, 'Math.ceil(');
    expr = expr.replace(/floor\(/g, 'Math.floor(');
    expr = expr.replace(/round\(/g, 'Math.round(');
    expr = expr.replace(/log\(/g, 'Math.log10(');
    expr = expr.replace(/ln\(/g, 'Math.log(');

    var result = eval(expr); // 在 QuickJS 沙箱中执行

    if (typeof result !== 'number' || !isFinite(result)) {
      return { error: '计算结果无效' };
    }

    // 格式化输出（避免浮点精度问题）
    var formatted = Math.abs(result) < 0.0001 || Math.abs(result) > 1e15 ? result.toExponential(6) : parseFloat(result.toPrecision(12));

    return { result: formatted };
  } catch (e) {
    return { error: '表达式错误: ' + e.message };
  }
}

// ==================== 单位换算 ====================

var UNITS = {
  length: {
    m: 1, km: 1000, cm: 0.01, mm: 0.001,
    in: 0.0254, ft: 0.3048, yd: 0.9144,
    mile: 1609.344, nmi: 1852
  },
  weight: {
    kg: 1, g: 0.001, mg: 1e-6, t: 1000,
    lb: 0.453592, oz: 0.0283495
  },
  temperature: {
    c: 'c', f: 'f', k: 'k'
  },
  storage: {
    B: 1, KB: 1024, MB: 1048576, GB: 1073741824,
    TB: 1099511627776, PB: 1125899906842624
  }
};

function convertUnit(value, fromUnit, toUnit, category) {
  var units = UNITS[category];
  if (!units) return { error: '不支持的换算类别: ' + category };

  if (!units[fromUnit]) return { error: '不支持的单位: ' + fromUnit };
  if (!units[toUnit]) return { error: '不支持的单位: ' + toUnit };

  // 温度特殊处理
  if (category === 'temperature') {
    return convertTemp(value, fromUnit, toUnit);
  }

  var baseValue = value * units[fromUnit]; // 转为基准单位
  return { result: baseValue / units[toUnit] };
}

function convertTemp(value, from, to) {
  var celsius;
  switch (from.toLowerCase()) {
    case 'c': celsius = value; break;
    case 'f': celsius = (value - 32) * 5 / 9; break;
    case 'k': celsius = value - 273.15; break;
    default: return { error: '未知温度单位' };
  }
  switch (to.toLowerCase()) {
    case 'c': return { result: celsius };
    case 'f': return { result: celsius * 9 / 5 + 32 };
    case 'k': return { result: celsius + 273.15 };
    default: return { error: '未知温度单位' };
  }
}

// ==================== 百分比计算 ====================

function calcPercent(type, values) {
  var a = values[0], b = values[1];
  switch (type) {
    case 'of':
      // A 的 B% 是多少
      return { result: a * b / 100, label: a + ' 的 ' + b + '% = ' };
    case 'is':
      // A 是 B 的多少%
      return { result: b !== 0 ? (a / b) * 100 : null, label: a + ' 是 ' + b + ' 的 ', suffix: '%' };
    case 'increase':
      // 从 A 增加到 B，增幅%
      return { result: a !== 0 ? ((b - a) / a) * 100 : null, label: '从 ' + a + ' 到 ' + b + ' 增幅', suffix: '%' };
    case 'decrease':
      // 从 A 减少到 B，降幅%
      return { result: a !== 0 ? ((a - b) / a) * 100 : null, label: '从 ' + a + ' 到 ' + b + ' 降幅', suffix: '%' };
    default:
      return { error: '未知的百分比类型' };
  }
}

// ==================== 常用公式参考 ====================

var FORMULAS = [
  { name: '圆面积', formula: 'S = \u03C0r\u00B2', desc: '半径 r 的圆面积' },
  { name: '圆周长', formula: 'C = 2\u03C0r', desc: '' },
  { name: '球体积', formula: 'V = 4/3 \u03C0r\u00B3', desc: '' },
  { name: '勾股定理', formula: 'c\u00B2 = a\u00B2 + b\u00B2', desc: '直角三角形斜边' },
  { name: '等差数列求和', formula: 'Sn = n(a1+an)/2', desc: '' },
  { name: '等比数列求和', formula: 'Sn = a1(1-q\u207F)/(1-q)', desc: 'q\u22601' },
  { name: '二次方程', formula: 'x = (-b\u00B1\u221A(b\u00B2-4ac))/2a', desc: '' },
  { name: '利息(复利)', formula: 'A = P(1+r/n)^(nt)', desc: '' },
  { name: '速度', formula: 'v = s/t', desc: '路程 / 时间' },
  { name: '密度', formula: '\u03C1 = m/V', desc: '质量 / 体积' }
];

// ==================== 插件生命周期 ====================

function pluginInit() {
  console.log('[calc] ✅ 计算器插件就绪');
  return { success: true, message: 'Calculator ready!' };
}

// ==================== 搜索函数 ====================

function onSearch(query) {
  console.log('[calc] 🔍 收到搜索请求:', query);
  var results = [];
  var q = query.toLowerCase().trim();

  // 空查询或匹配前缀时显示全部功能入口
  if (!q || q === '' || q.indexOf('calc') === 0 || q.indexOf('计算') >= 0 || q.indexOf('数学') >= 0) {
    results.push({ title: '🔢 表达式计算', description: '输入数学表达式求值，如 2+3*4, sin(pi/2), sqrt(144)', icon: '🔢', action: 'evalExpr' });
    results.push({ title: '📐 长度换算', description: 'm/km/cm/in/ft/mile 互转', icon: '📐', action: 'unitConvert', data: { category: 'length' } });
    results.push({ title: '⚖️ 重量换算', description: 'kg/g/lb/oz 互转', icon: '⚖️', action: 'unitConvert', data: { category: 'weight' } });
    results.push({ title: '🌡️ 温度换算', description: '℃/℉/K 互转', icon: '🌡️', action: 'unitConvert', data: { category: 'temperature' } });
    results.push({ title: '💾 存储换算', description: 'B/KB/MB/GB/TB/PB 互转', icon: '💾', action: 'unitConvert', data: { category: 'storage' } });
    results.push({ title: '📊 百分比计算', description: '求占比、增幅、降幅', icon: '📊', action: 'percentCalc' });
    results.push({ title: '🧮 公式速查', description: '常用数学公式参考', icon: '🧮', action: 'formulas' });

    // 如果输入看起来像数学表达式，直接提供快速计算
    if (q.length > 1 && /^[\d\s+\-*/().^%\w]+$/.test(q) && !q.match(/[a-df-z]/i)) {
      var evalResult = safeEval(q);
      if (evalResult.result !== undefined) {
        results.unshift({
          title: '⚡ = ' + evalResult.result,
          description: '点击复制结果',
          icon: '=',
          action: 'quickEval',
          data: { input: q, result: evalResult.result }
        });
      }
    }
  }

  // 关键词匹配特定功能
  if (q.indexOf('pi') >= 0 || q.indexOf('\u03C0') >= 0 || q.indexOf('圆') >= 0) {
    results.push({ title: '\u03C0 = 3.14159265...', description: '圆周率常量', icon: '🔴', action: 'constPi' });
  }
  if (q.indexOf('e ') >= 0 && q.length < 5) {
    results.push({ title: 'e = 2.71828182...', description: '自然对数底', icon: 'e', action: 'constE' });
  }

  return results.slice(0, 10);
}

// ==================== 动作执行 ====================

function onAction(action, data) {
  console.log('[calc] ⚡ 执行动作:', action, data);

  switch (action) {
    case 'evalExpr':
      return { type: 'text', message: '🔢 表达式计算\n\n支持的运算:\n• 四则运算: 2+3*4, (1+2)*3\n• 幂次: 2^10, 16^0.5\n• 数学函数: sin(pi/3), sqrt(144)\n• 常量: pi, e\n\n输入表达式即可自动计算', hint: '如: 2*(3+4)^2 或 15% of 200' };

    case 'quickEval':
      try {
        utools.clipboard.writeText(String(data.result));
        return { type: 'text', message: '✅ ' + data.input + ' = ' + data.result + '\n\n结果已复制到剪贴板' };
      } catch (e) {
        return { type: 'error', message: '❌ 复制失败: ' + e.message };
      }

    case 'constPi':
      utools.clipboard.writeText(String(Math.PI));
      return { type: 'text', message: '✅ π ≈ 3.141592653589793\n\n已复制' };

    case 'constE':
      utools.clipboard.writeText(String(Math.E));
      return { type: 'text', message: '✅ e ≈ 2.718281828459045\n\n已复制' };

    case 'unitConvert':
      var cat = data.category || 'length';
      var unitNames = Object.keys(UNITS[cat]);
      return { type: 'text', message: '📐 ' + cat.toUpperCase() + ' 换算\n\n可用单位: ' + unitNames.join(', ') + '\n\n使用格式: 数值 单位 → 目标单位\n例: 100 cm m (100厘米=多少米)', hint: '如: 100 km m 或 32 ℃ ℉' };

    case 'percentCalc':
      return { type: 'text', message: '📊 百分比计算\n\n支持的模式:\n• "X 是 Y 的百分之几" (is)\n• "X 的 Y%" (of)\n• "从 X 到 Y 增幅" (increase)\n• "从 X 到 Y 降幅" (decrease)\n\n提示: 直接搜索 "50 is 200"', hint: '如: 200 的 15%' };

    case 'formulas':
      var formulaList = FORMULAS.map(function(f) {
        return '• ' + f.name + ': ' + f.formula + (f.desc ? ' (' + f.desc + ')' : '');
      }).join('\n');
      return { type: 'text', message: '🧮 常用数学公式\n\n' + formulaList };

    default:
      return { type: 'error', message: '❓ 未知动作: ' + action };
  }
}

// 导出
if (typeof module !== 'undefined' && module.exports) {
  module.exports = { pluginInit, onSearch, onAction };
}
