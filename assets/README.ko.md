<p align="center">
  <img src="toonify.png" alt="Toonify Logo" width="400">
</p>

# TOON (Token-Oriented Object Notation)

[English](../README.md) | [中文](README.zh-CN.md) | [한국어](README.ko.md)

구조화된 데이터를 대규모 언어 모델에 전달할 때 토큰 사용량을 크게 줄이도록 설계된 간결하고 사람이 읽을 수 있는 직렬화 형식입니다.

[![Python Version](https://img.shields.io/badge/python-3.8+-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 개요

TOON은 **CSV 수준의 간결함**을 달성하면서 **명시적인 구조**를 추가하여 다음과 같은 용도에 이상적입니다:
- LLM API 호출 시 토큰 비용 절감
- 컨텍스트 윈도우 효율성 향상
- 사람이 읽을 수 있는 형식 유지
- 데이터 구조 및 타입 보존

### 주요 기능

- ✅ **간결함**: 구조화된 데이터의 경우 JSON보다 30-60% 작음
- ✅ **가독성**: 깔끔하고 들여쓰기 기반의 구문
- ✅ **구조화**: 중첩된 객체와 배열 보존
- ✅ **타입 안전성**: 문자열, 숫자, 불리언, null 지원
- ✅ **유연성**: 다양한 구분자 옵션 (쉼표, 탭, 파이프)
- ✅ **스마트**: 균일한 배열을 위한 자동 테이블 형식
- ✅ **효율성**: 깊게 중첩된 객체를 위한 키 폴딩

## 설치

```bash
pip install toonify
```

개발 환경:
```bash
pip install toonify[dev]
```

## 빠른 시작

### Python API

```python
from toon import encode, decode

# Python dict를 TOON으로 인코딩
data = {
    'products': [
        {'sku': 'LAP-001', 'name': 'Gaming Laptop', 'price': 1299.99},
        {'sku': 'MOU-042', 'name': 'Wireless Mouse', 'price': 29.99}
    ]
}

toon_string = encode(data)
print(toon_string)
# 출력:
# products[2]{sku,name,price}:
#   LAP-001,Gaming Laptop,1299.99
#   MOU-042,Wireless Mouse,29.99

# TOON을 다시 Python으로 디코딩
result = decode(toon_string)
assert result == data
```

### 명령줄

```bash
# JSON을 TOON으로 인코딩
toon input.json -o output.toon

# TOON을 JSON으로 디코딩
toon input.toon -o output.json

# 파이프 사용
cat data.json | toon -e > data.toon

# 토큰 통계 표시
toon data.json --stats
```

## TOON 형식 사양

### 기본 구문

```toon
# 간단한 키-값 쌍
title: Machine Learning Basics
chapters: 12
published: true
```

### 배열

**기본 배열** (인라인):
```toon
temperatures: [72.5,68.3,75.1,70.8,73.2]
categories: [electronics,computers,accessories]
```

**테이블 배열** (헤더가 있는 균일한 객체):
```toon
inventory[3]{sku,product,stock}:
  KB-789,Mechanical Keyboard,45
  MS-456,RGB Mouse Pad,128
  HD-234,USB Headset,67
```

**리스트 배열** (불균일하거나 중첩된):
```toon
tasks[2]:
  Complete documentation
  Review pull requests
```

### 중첩 객체

```toon
server:
  hostname: api-prod-01
  config:
    port: 8080
    region: us-east
```

### 따옴표 규칙

문자열은 필요한 경우에만 따옴표로 묶습니다:
- 특수 문자 포함 (`,`, `:`, `"`, 줄바꿈)
- 앞/뒤 공백 있음
- 리터럴처럼 보임 (`true`, `false`, `null`)
- 비어있음

```toon
simple: ProductName
quoted: "Product, Description"
escaped: "Size: 15\" display"
multiline: "First feature\nSecond feature"
```

## API 레퍼런스

### `encode(data, options=None)`

Python 객체를 TOON 문자열로 변환합니다.

**매개변수:**
- `data`: Python dict 또는 list
- `options`: 선택적 dict:
  - `delimiter`: `'comma'` (기본값), `'tab'`, 또는 `'pipe'`
  - `indent`: 레벨당 공백 수 (기본값: 2)
  - `key_folding`: `'off'` (기본값) 또는 `'safe'`
  - `flatten_depth`: 키 폴딩의 최대 깊이 (기본값: None)

**예제:**
```python
toon = encode(data, {
    'delimiter': 'tab',
    'indent': 4,
    'key_folding': 'safe'
})
```

### `decode(toon_string, options=None)`

TOON 문자열을 Python 객체로 변환합니다.

**매개변수:**
- `toon_string`: TOON 형식 문자열
- `options`: 선택적 dict:
  - `strict`: 구조를 엄격하게 검증 (기본값: True)
  - `expand_paths`: `'off'` (기본값) 또는 `'safe'`
  - `default_delimiter`: 기본 구분자 (기본값: `','`)

**예제:**
```python
data = decode(toon_string, {
    'expand_paths': 'safe',
    'strict': False
})
```

## CLI 사용법

```
usage: toon [-h] [-o OUTPUT] [-e] [-d] [--delimiter {comma,tab,pipe}]
            [--indent INDENT] [--stats] [--no-strict]
            [--key-folding {off,safe}] [--flatten-depth DEPTH]
            [--expand-paths {off,safe}]
            [input]

TOON (Token-Oriented Object Notation) - JSON과 TOON 형식 간 변환

positional arguments:
  input                 입력 파일 경로 (또는 stdin의 경우 "-")

optional arguments:
  -h, --help            도움말 메시지 표시
  -o, --output OUTPUT   출력 파일 경로 (기본값: stdout)
  -e, --encode          인코딩 모드 강제 (JSON에서 TOON으로)
  -d, --decode          디코딩 모드 강제 (TOON에서 JSON으로)
  --delimiter {comma,tab,pipe}
                        배열 구분자 (기본값: comma)
  --indent INDENT       들여쓰기 크기 (기본값: 2)
  --stats               토큰 통계 표시
  --no-strict           엄격한 검증 비활성화 (디코딩만)
  --key-folding {off,safe}
                        키 폴딩 모드 (인코딩만)
  --flatten-depth DEPTH 최대 키 폴딩 깊이 (인코딩만)
  --expand-paths {off,safe}
                        경로 확장 모드 (디코딩만)
```

## 고급 기능

### 키 폴딩

단일 키 체인을 점으로 구분된 경로로 축소합니다:

```python
data = {
    'api': {
        'response': {
            'product': {
                'title': 'Wireless Keyboard'
            }
        }
    }
}

# key_folding='safe' 사용
toon = encode(data, {'key_folding': 'safe'})
# 출력: api.response.product.title: Wireless Keyboard
```

### 경로 확장

점으로 구분된 키를 중첩 객체로 확장합니다:

```python
toon = 'store.location.zipcode: 10001'

# expand_paths='safe' 사용
data = decode(toon, {'expand_paths': 'safe'})
# 결과: {'store': {'location': {'zipcode': 10001}}}
```

### 사용자 정의 구분자

데이터에 가장 적합한 구분자를 선택하세요:

```python
# 탭 구분자 (스프레드시트 같은 데이터에 더 좋음)
toon = encode(data, {'delimiter': 'tab'})

# 파이프 구분자 (데이터에 쉼표가 포함된 경우)
toon = encode(data, {'delimiter': 'pipe'})
```

## 형식 비교

### JSON vs TOON

**JSON** (247 바이트):
```json
{
  "products": [
    {"id": 101, "name": "Laptop Pro", "price": 1299},
    {"id": 102, "name": "Magic Mouse", "price": 79},
    {"id": 103, "name": "USB-C Cable", "price": 19}
  ]
}
```

**TOON** (98 바이트, **60% 감소**):
```toon
products[3]{id,name,price}:
  101,Laptop Pro,1299
  102,Magic Mouse,79
  103,USB-C Cable,19
```

### TOON 사용 시기

**TOON 사용:**
- ✅ LLM API에 데이터 전달 시 (토큰 비용 절감)
- ✅ 균일한 테이블 데이터 작업
- ✅ 컨텍스트 윈도우가 제한적일 때
- ✅ 사람이 읽을 수 있어야 할 때

**JSON 사용:**
- ❌ 최대 호환성이 필요할 때
- ❌ 데이터가 매우 불규칙하거나 중첩될 때
- ❌ 기존 JSON 전용 도구와 작업할 때

## 개발

### 설정

```bash
git clone https://github.com/ScrapeGraphAI/toonify.git
cd toonify
pip install -e .[dev]
```

### 테스트 실행

```bash
pytest
pytest --cov=toon --cov-report=term-missing
```

### 예제 실행

```bash
python examples/basic_usage.py
python examples/advanced_features.py
```

## 성능

TOON은 일반적으로 다음을 달성합니다:
- 구조화된 데이터의 경우 JSON 대비 **30-60% 크기 감소**
- 테이블 데이터의 경우 **40-70% 토큰 감소**
- 인코딩/디코딩 시 **최소 오버헤드** (일반적인 페이로드의 경우 <1ms)

## 기여

기여를 환영합니다! 다음 단계를 따라주세요:

1. 저장소 포크
2. 기능 브랜치 생성 (`git checkout -b feature/amazing-feature`)
3. 테스트와 함께 변경 사항 작성
4. 테스트 실행 (`pytest`)
5. 변경 사항 커밋 (`git commit -m 'Add amazing feature'`)
6. 브랜치에 푸시 (`git push origin feature/amazing-feature`)
7. Pull Request 열기

## 라이선스

MIT 라이선스 - 자세한 내용은 [LICENSE](../LICENSE) 파일을 참조하세요.

## 크레딧

Python 구현은 [toon-format/toon](https://github.com/toon-format/toon)의 TypeScript TOON 라이브러리에서 영감을 받았습니다.

## 링크

- **GitHub**: https://github.com/ScrapeGraphAI/toonify
- **PyPI**: https://pypi.org/project/toonify/
- **문서**: https://github.com/ScrapeGraphAI/toonify#readme
- **형식 사양**: https://github.com/toon-format/toon

---

[ScrapeGraph 팀](https://scrapegraphai.com)이 ❤️으로 만들었습니다

<p align="center">
  <img src="https://github.com/ScrapeGraphAI/Scrapegraph-ai/blob/main/docs/assets/scrapegraphai_logo.png" alt="ScrapeGraphAI Logo" width="250">
</p>

