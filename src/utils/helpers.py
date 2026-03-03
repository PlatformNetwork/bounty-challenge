def process_data(input_list):
    """处理数据列表，过滤None值并转换为整数。"""
    if not isinstance(input_list, list):
        raise TypeError("输入必须是列表类型")
    
    # 过滤None值并转换为整数
    result = []
    for item in input_list:
        if item is not None:
            try:
                result.append(int(item))
            except (ValueError, TypeError):
                # 跳过无法转换的值而非中断
                continue
    return result