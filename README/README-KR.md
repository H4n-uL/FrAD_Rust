# Fourier Analogue-in-Digital

## 프로젝트 개요

[AAPM](https://mikhael-openworkspace.notion.site/Project-Archivist-e512fa7a21474ef6bdbd615a424293cf)@Audio-8151의 Rust 구현체입니다. 자세한 내용은 [Notion](https://mikhael-openworkspace.notion.site/Fourier-Analogue-in-Digital-d170c1760cbf4bb4aaea9b1f09b7fead?pvs=4)에서 확인하실 수 있습니다.

## 입출력 PCM 포맷

부동소수점

- f16be, f32be, f64be(기본값)
- f16le, f32le, f64le

부호 있는 정수

- s8
- s16be, s24be, s32be, s64be
- s16le, s24le, s32le, s64le

부호 없는 정수

- u8
- u16be, u24be, u32be, u64be
- u16le, u24le, u32le, u64le

## 설치 방법

1. Git clone으로 라이브러리 다운로드
2. cargo build --release로 빌드
3. target/release에 있는 실행 파일을 원하는 위치로 이동
4. PATH에 변수 등록

```bash
git clone https://github.com/h4n-ul/FrAD_Rust.git
cd FrAD_Rust
cargo build --release
mv target/release/frad /path/to/bin/frad
export PATH=/path/to/bin:$PATH
```

**경고: `--release`를 빼두고 빌드하면 실행 속도가 극도로 느려지므로 꼭 `--release`를 붙이고 빌드해주시기 바랍니다.**

## 메타데이터 JSON 예시

```json
[
    {"key": "키",                  "type": "string", "value": "값"},
    {"key": "원작자",               "type": "string", "value": "한울"},
    {"key": "키와 String타입 인코딩", "type": "string", "value": "UTF-8"},
    {"key": "Base64 지원",         "type": "base64", "value": "QmFzZTY0IOyYiOyLnA=="},
    {"key": "파일 지원",            "type": "base64", "value": "7LWc64yAIDI1NlRpQuq5jOyngCDsp4Dsm5A="},
    {"key": "미지원 글자 없음",       "type": "string", "value": "유니코드에 있는 어떤 글자라도 호환됩니다!"},
    {"key": "중복 키 지원",          "type": "string", "value": "중복 키를 넣으면?"},
    {"key": "중복 키 지원",          "type": "string", "value": "짠!"},
    {"key": "",                   "type": "string", "value": "키 없는 메타데이터도 지원"}
]
```

## 외부 리소스

[Rust](https://github.com/rust-lang/rust)

### Cargo 크레이트

1. flate2
2. half
3. libsoxr
4. rustfft

## 기여 방법

### FrAD 포맷에 대한 기여

FrAD 포맷 자체에 대한 기여는 [여기](https://github.com/h4n-ul/Fourier_Analogue-in-Digital)에 해주시거나 저에게 직접 메일을 작성하셔야 합니다. 표준에 대한 제안, 개선사항 등은 링크에 있는 레포지토리에 이슈나 PR을 생성해주시기 바랍니다.

### Rust 구현체에 대한 기여

Rust 구현체에 한정되는 기여라면 이 레포지토리에 직접 해주시면 됩니다. 기능 추가든 버그 수정이든 성능 개선이든 어떤거든 다 환영입니다.

다음은 기여 절차입니다.

1. 레포지토리 포크하기
2. 새로운 브랜치 생성하기
3. 수정 사항을 만들고 버그에 고통받기
4. main 브랜치에 푸시하기
5. 이 레포지토리로 Pull Request 생성하기

Pull Request가 생성되면 검토 후 피드백을 드리거나 merge 하도록 하겠습니다. 사실 FrAD 표준과 호환되면 대부분 덥썩 물어드립니다.

## 개발자 정보

한울, <jun061119@proton.me>
