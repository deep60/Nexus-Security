# ML models

The analysis-engine ML analyzer (cargo feature `ml-engine`) loads two optional
ONNX models from this directory:

| File                    | Role                          | Env override          |
|-------------------------|-------------------------------|-----------------------|
| `threat_classifier.onnx`| Multi-class threat classifier | `ML_CLASSIFIER_MODEL` |
| `anomaly_detector.onnx` | Anomaly score (single output) | `ML_ANOMALY_MODEL`    |

> ⚠️ The files currently committed here are **empty placeholders (0 bytes)**.
> Until real trained models are provided, the ML analyzer loads in a
> degraded state: it returns a non-fatal `Unknown` detection (`error_message:
> "ML models not available"`) and never aborts a scan.

## Model I/O contract

The analyzer feeds each model a single input tensor and reads a single output
tensor (bound positionally, so input/output *names* don't matter):

- **Input:** `float32`, shape `[1, feature_size]` (`feature_size` defaults to
  **259**, configurable via `ML_FEATURE_SIZE`).
- **Classifier output:** `float32`, shape `[1, n_classes]` — a per-class score
  vector. `argmax` selects the class; index 0 is treated as `benign`. The label
  list is index-aligned (see `MlAnalyzerConfig::labels`).
- **Anomaly output:** `float32`, a single score compared against
  `ML_ANOMALY_THRESHOLD` (default `0.5`).

### Feature vector layout

`MlAnalyzer::extract_features` produces, in order:

```
[ size_norm, entropy/8, printable_ratio, <256-bin byte-frequency histogram> ]
   \_____________ 3 scalars ______________/  \________ 256 bins ________/
```

which is `3 + 256 = 259` floats — the default `feature_size`, so the layout is
passed through intact. **A model must be trained against this exact layout** (or
update `extract_features` to match your training pipeline).

> Setting `ML_FEATURE_SIZE` *below* 259 truncates the tail of the histogram
> (the analyzer logs a warning at load time saying how many bins survive);
> setting it above 259 zero-pads. Only do either if your model was trained on
> the corresponding shape.
>
> Historical note: `feature_size` defaulted to 256 while the extractor emitted
> 259 values, so histogram bins 253-255 were dropped without warning. Models
> trained against that old truncated layout need `ML_FEATURE_SIZE=256` or a
> retrain.

## Enabling ML in a build

ML is intentionally **off** in the default Docker build because no real models
ship. To enable it:

1. Drop valid `.onnx` models into this directory (or set the env overrides).
2. Build the engine with the feature, e.g.
   `cargo build --release --bin analysis-engine --features clamav,yara-engine,ml-engine`
   (or `--features native-engines`).
3. Ensure the ONNX Runtime shared library is available in the runtime image.
   `ort` downloads it at build time; for a slim runtime container, copy the
   `libonnxruntime.so*` from the builder stage or install it explicitly.

Configuration env vars: `ENABLE_ML_ENGINE`, `ML_CLASSIFIER_MODEL`,
`ML_ANOMALY_MODEL`, `ML_FEATURE_SIZE`, `ML_ANOMALY_THRESHOLD`.
