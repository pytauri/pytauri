from itertools import chain
from json import dumps
from typing import Annotated, Literal, Optional

from pydantic import BaseModel, Field
from pydantic.json_schema import JsonSchemaValue, models_json_schema

Model = type[BaseModel]
ModelValidation = tuple[Model, Literal["validation"]]
ModelSerialization = tuple[Model, Literal["serialization"]]

CommandInputOutput = dict[str, tuple[Optional[Model], Optional[Model]]]
CommandInputOutputWithMode = dict[
    str,
    tuple[
        Optional[ModelValidation],
        Optional[ModelSerialization],
    ],
]


def gen_json_schame(cmd_io: CommandInputOutput) -> JsonSchemaValue:
    cmd_io_with_mode: CommandInputOutputWithMode = {
        cmd: (
            (input_model, "validation") if input_model else None,
            (output_model, "serialization") if output_model else None,
        )
        for cmd, (input_model, output_model) in cmd_io.items()
    }

    json_schemas_map, definitions = models_json_schema(
        tuple(filter(None, chain.from_iterable(cmd_io_with_mode.values())))
    )

    bytes_ts_type = {
        "tsType": "ArrayBuffer",
    }

    json_schemas = {
        "title": "Commands",
        "type": "object",
        "properties": {
            cmd: {
                "type": "array",
                "maxItems": 2,
                "minItems": 2,
                "items": [
                    json_schemas_map[input_model] if input_model else bytes_ts_type,
                    json_schemas_map[output_model] if output_model else bytes_ts_type,
                ],
            }
            for cmd, (input_model, output_model) in cmd_io_with_mode.items()
        },
        "required": list(cmd_io_with_mode.keys()),
        "additionalProperties": False,
        "$defs": definitions["$defs"],
    }

    return json_schemas


class Foo(BaseModel):
    a: int


class Bar(BaseModel):
    a: Annotated[int, Field(serialization_alias="a_s")]
    b: Foo


json_schemas = gen_json_schame(
    {
        "commandA": (Foo, None),
        "commandB": (Bar, Bar),
    }
)
print(dumps(json_schemas, indent=2))
