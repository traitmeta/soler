CREATE TABLE `scanner_height` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '主键id',
    `task_name` varchar(30) NOT NULL DEFAULT '' COMMENT '扫块任务名称',
    `chain_name` varchar(10) NOT NULL DEFAULT '' COMMENT '链名',
    `height` bigint(20) unsigned NOT NULL DEFAULT '0' COMMENT '当前高度',
    `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uniq_task_chain` (`task_name`, `chain_name`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8 COMMENT = '扫块高度表';

-- 合约事件表
CREATE TABLE `scanner_contract` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '主键id',
    `chain_name` varchar(10) NOT NULL DEFAULT '' COMMENT '链名',
    `chain_id` int(11) unsigned NOT NULL DEFAULT '0' COMMENT '当前高度',
    `address` varchar(66) NOT NULL DEFAULT '' COMMENT '扫描合约的地址',
    `event_sign` varchar(256) NOT NULL DEFAULT '' COMMENT '扫块事件的签名',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uniq_task_chain` (
        `chain_name`,
        `chain_id`,
        `address`,
        `event_sign`
    )
) ENGINE = InnoDB DEFAULT CHARSET = utf8 COMMENT = '扫块合约事件表';

CREATE TABLE `user` (
    `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT COMMENT '主键id',
    `user_name` varchar(20) NOT NULL DEFAULT '' COMMENT '用户名',
    `user_address` varchar(66) NOT NULL DEFAULT '' COMMENT '用户钱包地址',
    `user_email` varchar(66) NOT NULL DEFAULT '' COMMENT '用户邮箱',
    `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uniq_user_name` (`user_name`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8 COMMENT = '用户信息表';