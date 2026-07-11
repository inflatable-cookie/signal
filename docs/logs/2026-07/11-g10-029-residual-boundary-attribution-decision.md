# g10.029 Residual Boundary Attribution Decision

Date: 2026-07-11
Status: complete raw-bank reassessment selected

## Result

| Operator | Global condition |
| --- | ---: |
| full | `2.0862893665` |
| DC diagonalized | `2.0862893665` |
| preserved high edge diagonalized | `2.1170081614` |
| both boundaries diagonalized | `2.1170081614` |

The minimum mode at residue `3` carries `0.9815027396` Nyquist mass. The
maximum at residue `8` carries `0.9940580932`. DC contribution is numerically
zero in both. Preserved-high-edge cross removal raises the minimum-mode
Rayleigh by `0.0069501761` and lowers the maximum by only `0.0031271614`; it
does not close complete-bank conditioning.

Maximum errors are residual `4.0816637991e-13`, orthogonality
`9.0612880085e-15`, trace `1.0056895525e-15`, Frobenius
`1.2119742216e-14`, and closure `1.4078218646e-14`. Evidence hash
`a9f55eb001e8d125` repeats exactly.

## Decision

Boundary cross coupling is insufficient. Do not propose another endpoint
response, row allocation, delay set, or normalizer. Step back to a complete
raw-bank or transform-family reassessment.

## Next Task

Freeze Batch 29.6AD complete raw-bank reassessment.
