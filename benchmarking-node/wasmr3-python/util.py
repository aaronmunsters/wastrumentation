def parse_boolean(bool: str) -> bool:
    return {
        'True': True,
        'False': False,
    }[bool]
