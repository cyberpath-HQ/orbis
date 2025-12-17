/**
 * Form utilities tests
 * Tests for Zod schema building, form value initialization, and validation
 */

import {
    describe,
    it,
    expect,
    beforeEach
} from 'vitest';
import { z } from 'zod';

import {
    validationRuleToZod,
    buildFormSchema,
    getInitialFormValues,
    formatValidationErrors,
    hasValidationErrors,
    createFieldId,
    resetFieldIdCounter
} from '@/lib/form-utils';
import type { FieldSchema } from '@/types/schema/components';
import type { ValidationRule } from '@/types/schema/base';

describe(`validationRuleToZod`, () => {
    describe(`field type schemas`, () => {
        it(`should create number schema for number field`, () => {
            const schema = validationRuleToZod(undefined, `number`);

            expect(schema.safeParse(42).success).toBe(true);
            expect(schema.safeParse(`42`).success).toBe(true); // Coercion
        });

        it(`should create boolean schema for checkbox field`, () => {
            const schema = validationRuleToZod(undefined, `checkbox`);

            expect(schema.safeParse(true).success).toBe(true);
            expect(schema.safeParse(false).success).toBe(true);
        });

        it(`should create boolean schema for switch field`, () => {
            const schema = validationRuleToZod(undefined, `switch`);

            expect(schema.safeParse(true).success).toBe(true);
        });

        it(`should create email schema for email field`, () => {
            const schema = validationRuleToZod(undefined, `email`) as z.ZodString;

            expect(schema.safeParse(`test@example.com`).success).toBe(true);
            expect(schema.safeParse(`invalid-email`).success).toBe(false);
        });

        it(`should create url schema for url field`, () => {
            const schema = validationRuleToZod(undefined, `url`) as z.ZodString;

            expect(schema.safeParse(`https://example.com`).success).toBe(true);
            expect(schema.safeParse(`not-a-url`).success).toBe(false);
        });

        it(`should create string schema for text field`, () => {
            const schema = validationRuleToZod(undefined, `text`);

            expect(schema.safeParse(`hello`).success).toBe(true);
        });
    });

    describe(`required validation`, () => {
        it(`should enforce required for strings`, () => {
            const rule: ValidationRule = {
                required: true,
            };
            const schema = validationRuleToZod(rule, `text`);

            expect(schema.safeParse(``).success).toBe(false);
            expect(schema.safeParse(`value`).success).toBe(true);
        });

        it(`should use custom required message`, () => {
            const rule: ValidationRule = {
                required: {
                    message: `Name is required`,
                },
            };
            const schema = validationRuleToZod(rule, `text`);
            const result = schema.safeParse(``);

            expect(result.success).toBe(false);
            if (!result.success) {
                expect(result.error.issues[0].message).toBe(`Name is required`);
            }
        });

        it(`should make field optional when not required`, () => {
            const schema = validationRuleToZod(undefined, `text`);

            expect(schema.safeParse(undefined).success).toBe(true);
        });
    });

    describe(`string validations`, () => {
        it(`should validate minLength when required`, () => {
            const rule: ValidationRule = {
                required:  true,
                minLength: 5,
            };
            const schema = validationRuleToZod(rule, `text`);

            expect(schema.safeParse(`hi`).success).toBe(false);
            expect(schema.safeParse(`hello`).success).toBe(true);
        });

        it(`should validate minLength with custom message`, () => {
            const rule: ValidationRule = {
                required:  true,
                minLength: {
                    value:   5,
                    message: `Must be at least 5 characters`,
                },
            };
            const schema = validationRuleToZod(rule, `text`);
            const result = schema.safeParse(`hi`);

            expect(result.success).toBe(false);
            if (!result.success) {
                expect(result.error.issues[0].message).toBe(`Must be at least 5 characters`);
            }
        });

        it(`should validate maxLength when required`, () => {
            const rule: ValidationRule = {
                required:  true,
                maxLength: 10,
            };
            const schema = validationRuleToZod(rule, `text`);

            expect(schema.safeParse(`hello`).success).toBe(true);
            expect(schema.safeParse(`this is too long`).success).toBe(false);
        });

        it(`should validate pattern when required`, () => {
            const rule: ValidationRule = {
                required: true,
                pattern:  `^[A-Z]+$`,
            };
            const schema = validationRuleToZod(rule, `text`);

            expect(schema.safeParse(`ABC`).success).toBe(true);
            expect(schema.safeParse(`abc`).success).toBe(false);
        });

        it(`should validate pattern with custom message`, () => {
            const rule: ValidationRule = {
                required: true,
                pattern:  {
                    value:   `^[A-Z]+$`,
                    message: `Must be uppercase`,
                },
            };
            const schema = validationRuleToZod(rule, `text`);
            const result = schema.safeParse(`abc`);

            expect(result.success).toBe(false);
            if (!result.success) {
                expect(result.error.issues[0].message).toBe(`Must be uppercase`);
            }
        });
    });

    describe(`number validations`, () => {
        it(`should validate min when required`, () => {
            const rule: ValidationRule = {
                required: true,
                min:      0,
            };
            const schema = validationRuleToZod(rule, `number`);

            expect(schema.safeParse(-1).success).toBe(false);
            expect(schema.safeParse(0).success).toBe(true);
            expect(schema.safeParse(10).success).toBe(true);
        });

        it(`should validate max when required`, () => {
            const rule: ValidationRule = {
                required: true,
                max:      100,
            };
            const schema = validationRuleToZod(rule, `number`);

            expect(schema.safeParse(50).success).toBe(true);
            expect(schema.safeParse(100).success).toBe(true);
            expect(schema.safeParse(101).success).toBe(false);
        });

        it(`should validate min and max together`, () => {
            const rule: ValidationRule = {
                required: true,
                min:      1,
                max:      10,
            };
            const schema = validationRuleToZod(rule, `number`);

            expect(schema.safeParse(0).success).toBe(false);
            expect(schema.safeParse(5).success).toBe(true);
            expect(schema.safeParse(11).success).toBe(false);
        });
    });
});

describe(`buildFormSchema`, () => {
    it(`should build schema from field definitions`, () => {
        const fields: Array<FieldSchema> = [
            {
                type:      `Field`,
                id:        `field-name`,
                name:      `name`,
                fieldType: `text`,
                validation: {
                    required: true,
                },
            },
            {
                type:      `Field`,
                id:        `field-email`,
                name:      `email`,
                fieldType: `email`,
                validation: {
                    required: true,
                },
            },
            {
                type:      `Field`,
                id:        `field-age`,
                name:      `age`,
                fieldType: `number`,
                validation: {
                    min: 0,
                    max: 150,
                },
            },
        ];

        const schema = buildFormSchema(fields);

        // Valid data
        const validResult = schema.safeParse({
            name:  `John`,
            email: `john@example.com`,
            age:   30,
        });
        expect(validResult.success).toBe(true);

        // Invalid: missing required name
        const invalidResult = schema.safeParse({
            name:  ``,
            email: `john@example.com`,
            age:   30,
        });
        expect(invalidResult.success).toBe(false);
    });

    it(`should handle empty fields array`, () => {
        const schema = buildFormSchema([]);

        expect(schema.safeParse({}).success).toBe(true);
    });
});

describe(`getInitialFormValues`, () => {
    it(`should use field default values`, () => {
        const fields: Array<FieldSchema> = [
            {
                type:         `Field`,
                id:           `field-name`,
                name:         `name`,
                fieldType:    `text`,
                defaultValue: `Default Name`,
            },
            {
                type:         `Field`,
                id:           `field-count`,
                name:         `count`,
                fieldType:    `number`,
                defaultValue: `10`,
            },
        ];

        const values = getInitialFormValues(fields);

        expect(values.name).toBe(`Default Name`);
        expect(values.count).toBe(`10`);
    });

    it(`should use type-appropriate defaults when no defaultValue`, () => {
        const fields: Array<FieldSchema> = [
            {
                type:      `Field`,
                id:        `field-text`,
                name:      `text`,
                fieldType: `text`,
            },
            {
                type:      `Field`,
                id:        `field-num`,
                name:      `num`,
                fieldType: `number`,
            },
            {
                type:      `Field`,
                id:        `field-check`,
                name:      `check`,
                fieldType: `checkbox`,
            },
            {
                type:      `Field`,
                id:        `field-sw`,
                name:      `sw`,
                fieldType: `switch`,
            },
        ];

        const values = getInitialFormValues(fields);

        expect(values.text).toBe(``);
        expect(values.num).toBe(0);
        expect(values.check).toBe(false);
        expect(values.sw).toBe(false);
    });

    it(`should use state binding values when available`, () => {
        const fields: Array<FieldSchema> = [
            {
                type:      `Field`,
                id:        `field-name`,
                name:      `name`,
                fieldType: `text`,
                bindTo:    `user.name`,
            },
        ];

        const state = {
            user: {
                name: `John from state`,
            },
        };

        const values = getInitialFormValues(fields, state);

        expect(values.name).toBe(`John from state`);
    });

    it(`should prefer state binding over default value`, () => {
        const fields: Array<FieldSchema> = [
            {
                type:         `Field`,
                id:           `field-name`,
                name:         `name`,
                fieldType:    `text`,
                bindTo:       `name`,
                defaultValue: `Default`,
            },
        ];

        const state = {
            name: `State Value`,
        };

        const values = getInitialFormValues(fields, state);

        expect(values.name).toBe(`State Value`);
    });

    it(`should fall back to default when state value is undefined`, () => {
        const fields: Array<FieldSchema> = [
            {
                type:         `Field`,
                id:           `field-name`,
                name:         `name`,
                fieldType:    `text`,
                bindTo:       `missing.path`,
                defaultValue: `Fallback`,
            },
        ];

        const state = {};

        const values = getInitialFormValues(fields, state);

        expect(values.name).toBe(`Fallback`);
    });
});

describe(`formatValidationErrors`, () => {
    it(`should format errors map`, () => {
        const errors = {
            name:  [
                `Name is required`,
            ],
            email: [
                `Invalid email`,
                `Email already exists`,
            ],
        };

        const formatted = formatValidationErrors(errors);

        expect(formatted.get(`name`)).toBe(`Name is required`);
        expect(formatted.get(`email`)).toBe(`Invalid email`); // First error only
    });

    it(`should return empty map for undefined errors`, () => {
        const formatted = formatValidationErrors(undefined);

        expect(formatted.size).toBe(0);
    });

    it(`should skip fields with no errors`, () => {
        const errors = {
            name:  [],
            email: [
                `Invalid email`,
            ],
        };

        const formatted = formatValidationErrors(errors);

        expect(formatted.has(`name`)).toBe(false);
        expect(formatted.has(`email`)).toBe(true);
    });
});

describe(`hasValidationErrors`, () => {
    it(`should return true when there are errors`, () => {
        const errors = {
            name: [
                `Required`,
            ],
        };

        expect(hasValidationErrors(errors)).toBe(true);
    });

    it(`should return false when no errors`, () => {
        const errors = {
            name:  [],
            email: [],
        };

        expect(hasValidationErrors(errors)).toBe(false);
    });

    it(`should return false for undefined`, () => {
        expect(hasValidationErrors(undefined)).toBe(false);
    });
});

describe(`createFieldId`, () => {
    beforeEach(() => {
        resetFieldIdCounter();
    });

    it(`should create unique field IDs`, () => {
        const id1 = createFieldId();
        const id2 = createFieldId();
        const id3 = createFieldId();

        expect(id1).toBe(`field_1`);
        expect(id2).toBe(`field_2`);
        expect(id3).toBe(`field_3`);
    });

    it(`should reset counter`, () => {
        createFieldId();
        createFieldId();
        resetFieldIdCounter();

        expect(createFieldId()).toBe(`field_1`);
    });
});
