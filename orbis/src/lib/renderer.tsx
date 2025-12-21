/**
 * Plugin component renderer library
 * Renders JSON UI schemas using shadcn/ui components
 */

import React, {
    useMemo,
    useCallback,
    memo
} from 'react';
import { useNavigate } from 'react-router-dom';
import * as LucideIcons from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Checkbox } from '@/components/ui/checkbox';
import { Switch } from '@/components/ui/switch';
import {
    RadioGroup, RadioGroupItem
} from '@/components/ui/radio-group';
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue
} from '@/components/ui/select';
import {
    Tabs, TabsContent, TabsList, TabsTrigger
} from '@/components/ui/tabs';
import {
    Accordion, AccordionContent, AccordionItem, AccordionTrigger
} from '@/components/ui/accordion';
import {
    Alert, AlertDescription, AlertTitle
} from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import {
    Table, TableBody, TableCell, TableHead, TableHeader, TableRow
} from '@/components/ui/table';
import {
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle
} from '@/components/ui/dialog';
import {
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger
} from '@/components/ui/dropdown-menu';
import {
    Tooltip, TooltipContent, TooltipProvider, TooltipTrigger
} from '@/components/ui/tooltip';
import { Skeleton } from '@/components/ui/skeleton';
import {
    Avatar, AvatarFallback, AvatarImage
} from '@/components/ui/avatar';
import {
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator
} from '@/components/ui/breadcrumb';

import type {
    ComponentSchema,
    ContainerSchema,
    TextSchema,
    HeadingSchema,
    ButtonSchema,
    FieldSchema,
    FormSchema,
    TableSchema,
    CardSchema,
    ListSchema,
    ImageSchema,
    IconSchema,
    LinkSchema,
    BadgeSchema,
    AlertSchema,
    ProgressSchema,
    TabsSchema,
    AccordionSchema,
    ModalSchema,
    DropdownSchema,
    TooltipSchema,
    GridSchema,
    FlexSchema,
    SpacerSchema,
    DividerSchema,
    SkeletonSchema,
    AvatarSchema,
    BreadcrumbSchema,
    StatCardSchema,
    EmptyStateSchema,
    LoadingOverlaySchema,
    ConditionalSchema,
    LoopSchema,
    SectionSchema,
    PageHeaderSchema,
    DataDisplaySchema,
    Action
} from '../types/schema';
import {
    type PageStateStoreHook,
    getNestedValue,
    interpolateExpression,
    evaluateBooleanExpression
} from './state';
import {
    executeActions,
    type ActionContext,
    type ApiClient
} from './actions';
import { useForm } from '@tanstack/react-form';
import {
    buildFormSchema,
    getInitialFormValues
} from './form-utils';
import { extractAriaProps } from './a11y';
import { shallowEqual } from './performance';

// Form context for sharing form instance with field renderers
// We use `any` for the form instance type due to TanStack Form's complex generics
interface FormContextValue {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    form:     any
    formId:   string
    isInForm: boolean
}

const FormContext = React.createContext<FormContextValue>({
    form:     null,
    formId:   ``,
    isInForm: false,
});

function useFormContext(): FormContextValue {
    return React.useContext(FormContext);
}

// Renderer context
interface RendererContext {
    state:     PageStateStoreHook
    apiClient: ApiClient
    navigate:  ReturnType<typeof useNavigate>
    row?:      Record<string, unknown>
    item?:     unknown
    index?:    number
}

const RendererContextValue = React.createContext<RendererContext | null>(null);

function useRendererContext(): RendererContext {
    const ctx = React.useContext(RendererContextValue);
    if (!ctx) {
        throw new Error(`useRendererContext must be used within SchemaRenderer`);
    }
    return ctx;
}

// Main schema renderer
interface SchemaRendererProps {
    schema:    ComponentSchema
    state:     PageStateStoreHook
    apiClient: ApiClient
}

export function SchemaRenderer({
    schema,
    state,
    apiClient,
}: SchemaRendererProps): React.ReactElement | null {
    const navigate = useNavigate();

    const context = useMemo<RendererContext>(() => ({
        state,
        apiClient,
        navigate,
    }), [
        state,
        apiClient,
        navigate,
    ]);

    return (
        <RendererContextValue.Provider value={context}>
            <TooltipProvider>
                <ComponentRenderer schema={schema} />
            </TooltipProvider>
        </RendererContextValue.Provider>
    );
}

// Component renderer (memoized for performance)
interface ComponentRendererProps {
    schema: ComponentSchema
}

const ComponentRenderer = memo(function ComponentRenderer({
    schema,
}: ComponentRendererProps): React.ReactElement | null {
    const ctx = useRendererContext();
    // Use the state hook to subscribe to changes (reactive)
    const stateData = ctx.state((s) => s.state);

    if (!schema) {
        return null;
    }

    // Check visibility
    if (schema.visible !== undefined && schema.visible !== null) {
        const isVisible = evaluateBooleanExpression(schema.visible, stateData);
        if (!isVisible) {
            return null;
        }
    }

    console.log(`Rendering component:`, schema.type, `ID:`, schema.id);

    // Render based on type
    switch (schema.type) {
        case `Container`:
            return <ContainerRenderer schema={schema} />;
        case `Text`:
            return <TextRenderer schema={schema} />;
        case `Heading`:
            return <HeadingRenderer schema={schema} />;
        case `Button`:
            return <ButtonRenderer schema={schema} />;
        case `Field`:
            return <FieldRenderer schema={schema} />;
        case `Form`:
            return <FormRenderer schema={schema} />;
        case `Table`:
            return <TableRenderer schema={schema} />;
        case `Card`:
            return <CardRenderer schema={schema} />;
        case `List`:
            return <ListRenderer schema={schema} />;
        case `Image`:
            return <ImageRenderer schema={schema} />;
        case `Icon`:
            return <IconRenderer schema={schema} />;
        case `Link`:
            return <LinkRenderer schema={schema} />;
        case `Badge`:
            return <BadgeRenderer schema={schema} />;
        case `Alert`:
            return <AlertRenderer schema={schema} />;
        case `Progress`:
            return <ProgressRenderer schema={schema} />;
        case `Tabs`:
            return <TabsRenderer schema={schema} />;
        case `Accordion`:
            return <AccordionRenderer schema={schema} />;
        case `Modal`:
            return <ModalRenderer schema={schema} />;
        case `Dropdown`:
            return <DropdownRenderer schema={schema} />;
        case `Tooltip`:
            return <TooltipRenderer schema={schema} />;
        case `Grid`:
            return <GridRenderer schema={schema} />;
        case `Flex`:
            return <FlexRenderer schema={schema} />;
        case `Spacer`:
            return <SpacerRenderer schema={schema} />;
        case `Divider`:
            return <DividerRenderer schema={schema} />;
        case `Skeleton`:
            return <SkeletonRenderer schema={schema} />;
        case `Avatar`:
            return <AvatarRenderer schema={schema} />;
        case `Breadcrumb`:
            return <BreadcrumbRenderer schema={schema} />;
        case `StatCard`:
            return <StatCardRenderer schema={schema} />;
        case `EmptyState`:
            return <EmptyStateRenderer schema={schema} />;
        case `LoadingOverlay`:
            return <LoadingOverlayRenderer schema={schema} />;
        case `Conditional`:
            return <ConditionalRenderer schema={schema} />;
        case `Loop`:
            return <LoopRenderer schema={schema} />;
        case `Section`:
            return <SectionRenderer schema={schema} />;
        case `PageHeader`:
            return <PageHeaderRenderer schema={schema} />;
        case `DataDisplay`:
            return <DataDisplayRenderer schema={schema} />;
        case `Fragment`:
            return (
                <>
                    {schema.children.map((child, i) => (
                        <ComponentRenderer key={child.id ?? i} schema={child} />
                    ))}
                </>
            );
        case `Custom`:
        case `Chart`:
        case `Slot`:
            return (
                <div className="text-muted-foreground">
                    Component type &quot;{schema.type}&quot; not yet implemented
                </div>
            );
        default:
            return null;
    }
}, (prevProps, nextProps) => {
    // Custom comparison for memoization
    // Only re-render if schema reference changed or schema.id changed
    if (prevProps.schema === nextProps.schema) {
        return true;
    }
    if (prevProps.schema.id !== nextProps.schema.id) {
        return false;
    }

    // Use shallow comparison for schema object
    return shallowEqual(
        prevProps.schema as unknown as Record<string, unknown>,
        nextProps.schema as unknown as Record<string, unknown>
    );
});

// Event handler helper
function useEventHandler(actions?: Array<Action>): (event?: unknown) => Promise<void> {
    const ctx = useRendererContext();

    return useCallback(async(event?: unknown): Promise<void> => {
        if (!actions || actions.length === 0) {
            return;
        }

        const actionContext: ActionContext = {
            state:     ctx.state,
            navigate:  ctx.navigate,
            apiClient: ctx.apiClient,
            event,
            row:       ctx.row,
            item:      ctx.item,
            index:     ctx.index,
        };

        console.log(`Executing actions:`, event, `with context:`, actionContext);

        await executeActions(actions, actionContext);
    }, [
        actions,
        ctx,
    ]);
}

// Expression resolver helper
function useResolvedValue(expression: string | undefined): string {
    const ctx = useRendererContext();
    const stateData = ctx.state((s) => s.state);

    if (!expression) {
        return ``;
    }

    return interpolateExpression(expression, stateData, {
        state:  stateData,
        $row:   ctx.row,
        $item:  ctx.item,
        $index: ctx.index,
    });
}

// Icon helper
function getIcon(name: string): React.ComponentType<{ className?: string }> | null {
    const IconComponent = (LucideIcons as Record<string, unknown>)[
        name.charAt(0).toUpperCase() + name.slice(1)
    ] as React.ComponentType<{ className?: string }> | undefined;
    return IconComponent ?? null;
}

// Individual component renderers

function ContainerRenderer({
    schema,
}: { schema: ContainerSchema }): React.ReactElement {
    const ctx = useRendererContext();
    // Use the state hook to subscribe to changes (reactive)
    const stateData = ctx.state((s) => s.state);
    const handleClick = useEventHandler(schema.events?.onClick);

    const handleOnClick = (): void => {
        void handleClick();
    };

    // Extract ARIA props for accessibility
    const ariaProps = extractAriaProps(schema, stateData);

    return (
        <div
            id={schema.id}
            className={schema.className}
            style={schema.style as React.CSSProperties}
            onClick={schema.events?.onClick ? handleOnClick : undefined}
            role={ariaProps.role as string | undefined}
            aria-label={ariaProps[`aria-label`] as string | undefined}
            aria-labelledby={ariaProps[`aria-labelledby`] as string | undefined}
            aria-describedby={ariaProps[`aria-describedby`] as string | undefined}
            aria-hidden={ariaProps[`aria-hidden`] as boolean | undefined}
        >
            {schema.children.map((child, i) => (
                <ComponentRenderer key={child.id ?? i} schema={child} />
            ))}
        </div>
    );
}

function TextRenderer({
    schema,
}: { schema: TextSchema }): React.ReactElement {
    const content = useResolvedValue(schema.content);

    const variantClasses: Record<string, string> = {
        body:    ``,
        caption: `text-sm text-muted-foreground`,
        label:   `text-sm font-medium`,
        code:    `font-mono text-sm bg-muted px-1 py-0.5 rounded`,
        muted:   `text-muted-foreground`,
    };

    return (
        <span
            id={schema.id}
            className={`${ variantClasses[schema.variant ?? `body`] } ${ schema.className ?? `` }`}
            style={schema.style as React.CSSProperties}
        >
            {content}
        </span>
    );
}

function HeadingRenderer({
    schema,
}: { schema: HeadingSchema }): React.ReactElement {
    const text = useResolvedValue(schema.text);
    const Tag = `h${ schema.level ?? 1 }` as keyof React.JSX.IntrinsicElements;

    const levelClasses: Record<number, string> = {
        1: `text-4xl font-bold tracking-tight`,
        2: `text-3xl font-semibold tracking-tight`,
        3: `text-2xl font-semibold`,
        4: `text-xl font-semibold`,
        5: `text-lg font-medium`,
        6: `text-base font-medium`,
    };

    return (
        <Tag
            id={schema.id}
            className={`${ levelClasses[schema.level ?? 1] } ${ schema.className ?? `` }`}
            style={schema.style as React.CSSProperties}
        >
            {text}
        </Tag>
    );
}

function ButtonRenderer({
    schema,
}: { schema: ButtonSchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);
    const handleClick = useEventHandler(schema.events?.onClick);
    const label = useResolvedValue(schema.label);

    const handleOnClick = (): void => {
        void handleClick(`click`);
    };

    const isDisabled = schema.disabled
        ? evaluateBooleanExpression(schema.disabled, stateData)
        : false;
    const isLoading = schema.loading
        ? evaluateBooleanExpression(schema.loading, stateData)
        : false;

    const Icon = schema.icon ? getIcon(schema.icon) : null;

    // Map size to Button's expected values
    const getButtonSize = (): `default` | `sm` | `lg` | `icon` | null => {
        switch (schema.size) {
            case `xs`:
            case `sm`:
                return `sm`;
            case `lg`:
            case `xl`:
                return `lg`;
            case `md`:
            default:
                return `default`;
        }
    };

    // Extract ARIA props for accessibility
    const ariaProps = extractAriaProps(schema, {
        ...stateData,
        $loading: isLoading,
    });

    return (
        <Button
            id={schema.id}
            variant={schema.variant}
            size={getButtonSize()}
            disabled={isDisabled || isLoading}
            onClick={handleOnClick}
            className={schema.className}
            style={schema.style as React.CSSProperties}
            aria-label={ariaProps[`aria-label`] as string | undefined}
            aria-disabled={isDisabled || isLoading}
            aria-busy={isLoading}
            {...(ariaProps.role && {
                role: ariaProps.role as string,
            })}
        >
            {isLoading && <LucideIcons.Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {Icon && schema.iconPosition !== `right` && <Icon className="mr-2 h-4 w-4" />}
            {label}
            {Icon && schema.iconPosition === `right` && <Icon className="ml-2 h-4 w-4" />}
        </Button>
    );
}

function FieldRenderer({
    schema,
}: { schema: FieldSchema }): React.ReactElement {
    const ctx = useRendererContext();
    const formCtx = useFormContext();
    // Use the state hook to subscribe to changes (reactive)
    const stateData = ctx.state((s) => s.state);
    const handleChange = useEventHandler(schema.events?.onChange);

    const label = useResolvedValue(schema.label);
    const placeholder = useResolvedValue(schema.placeholder);
    const description = useResolvedValue(schema.description);

    const isDisabled = schema.disabled
        ? evaluateBooleanExpression(schema.disabled, stateData)
        : false;

    // Get value from form context if inside a form, otherwise from state
    const formFieldMeta = formCtx.isInForm && formCtx.form
        ? formCtx.form.state.fieldMeta[schema.name]
        : null;
    const formValue = formCtx.isInForm && formCtx.form
        ? formCtx.form.state.values[schema.name]
        : undefined;

    // Calculate value - prefer form value, then bound state value, then default
    let value: unknown;
    if (formCtx.isInForm && formValue !== undefined) {
        value = formValue;
    } else if (schema.bindTo) {
        value = getNestedValue(stateData, schema.bindTo);
    } else {
        value = schema.defaultValue;
    }

    console.log(`[FieldRenderer] Rendering field "${ schema.name }", bindTo: "${ schema.bindTo }", value:`, value, `stateData:`, stateData);

    // Get validation error from form if inside form context
    const fieldError = formFieldMeta?.errors?.[0]
        ? String(formFieldMeta.errors[0])
        : undefined;
    const isTouched = formFieldMeta?.isTouched ?? false;
    const showError = isTouched && fieldError;

    const onValueChange = (newValue: unknown): void => {
        console.log(`[FieldRenderer] onValueChange called for "${ schema.name }" with value:`, newValue);
        console.log(`[FieldRenderer] bindTo: "${ schema.bindTo }", isInForm: ${ formCtx.isInForm }`);

        // Update form state if inside form
        if (formCtx.isInForm && formCtx.form) {
            formCtx.form.setFieldValue(schema.name, newValue);
        }

        // Also update page state if bindTo is set
        if (schema.bindTo) {
            console.log(`[FieldRenderer] Calling setState for "${ schema.bindTo }"`);
            ctx.state.setState(schema.bindTo, newValue);
        }

        void handleChange({
            value: newValue,
        });
    };

    const onBlur = (): void => {
        // Trigger validation on blur if inside form
        if (formCtx.isInForm && formCtx.form) {
            void formCtx.form.validateField(schema.name, `blur`);
        }
    };

    const renderInput = (): React.ReactElement => {
        const errorClass = showError ? `border-destructive` : ``;

        switch (schema.fieldType) {
            case `textarea`:
                return (
                    <Textarea
                        id={schema.id}
                        name={schema.name}
                        placeholder={placeholder}
                        disabled={isDisabled}
                        required={schema.required}
                        value={String(value ?? ``)}
                        onChange={(e) => onValueChange(e.target.value)}
                        onBlur={onBlur}
                        className={errorClass}
                        aria-invalid={showError ? `true` : undefined}
                        aria-describedby={showError ? `${ schema.id }-error` : undefined}
                    />
                );

            case `checkbox`:
                return (
                    <div className="flex items-center space-x-2">
                        <Checkbox
                            id={schema.id}
                            name={schema.name}
                            disabled={isDisabled}
                            checked={Boolean(value)}
                            onCheckedChange={onValueChange}
                            aria-invalid={showError ? `true` : undefined}
                        />
                        {label && <Label htmlFor={schema.id}>{label}</Label>}
                    </div>
                );

            case `switch`:
                return (
                    <div className="flex items-center space-x-2">
                        <Switch
                            id={schema.id}
                            name={schema.name}
                            disabled={isDisabled}
                            checked={Boolean(value)}
                            onCheckedChange={onValueChange}
                            aria-invalid={showError ? `true` : undefined}
                        />
                        {label && <Label htmlFor={schema.id}>{label}</Label>}
                    </div>
                );

            case `radio`:
                return (
                    <RadioGroup
                        value={String(value ?? ``)}
                        onValueChange={onValueChange}
                        disabled={isDisabled}
                        aria-invalid={showError ? `true` : undefined}
                    >
                        {schema.options?.map((opt) => (
                            <div key={opt.value} className="flex items-center space-x-2">
                                <RadioGroupItem value={opt.value} id={`${ schema.id }-${ opt.value }`} />
                                <Label htmlFor={`${ schema.id }-${ opt.value }`}>{opt.label}</Label>
                            </div>
                        ))}
                    </RadioGroup>
                );

            case `select`:
                return (
                    <Select
                        value={String(value ?? ``)}
                        onValueChange={onValueChange}
                        disabled={isDisabled}
                    >
                        <SelectTrigger
                            className={errorClass}
                            aria-invalid={showError ? `true` : undefined}
                        >
                            <SelectValue placeholder={placeholder} />
                        </SelectTrigger>
                        <SelectContent>
                            {schema.options?.map((opt) => (
                                <SelectItem key={opt.value} value={opt.value} disabled={opt.disabled}>
                                    {opt.label}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                );

            default:
                return (
                    <Input
                        id={schema.id}
                        name={schema.name}
                        type={schema.fieldType}
                        placeholder={placeholder}
                        disabled={isDisabled}
                        required={schema.required}
                        readOnly={schema.readOnly}
                        value={String(value ?? ``)}
                        onChange={(e) => onValueChange(e.target.value)}
                        onBlur={onBlur}
                        className={errorClass}
                        aria-invalid={showError ? `true` : undefined}
                        aria-describedby={showError ? `${ schema.id }-error` : undefined}
                    />
                );
        }
    };

    if (schema.fieldType === `checkbox` || schema.fieldType === `switch`) {
        return (
            <div className={`space-y-2 ${ schema.className ?? `` }`} style={schema.style as React.CSSProperties}>
                {renderInput()}
                {description && <p className="text-sm text-muted-foreground">{description}</p>}
                {showError && (
                    <p id={`${ schema.id }-error`} className="text-sm text-destructive" role="alert">
                        {fieldError}
                    </p>
                )}
            </div>
        );
    }

    return (
        <div className={`space-y-2 ${ schema.className ?? `` }`} style={schema.style as React.CSSProperties}>
            {label && (
                <Label htmlFor={schema.id} className={showError ? `text-destructive` : undefined}>
                    {label}
                    {schema.required && <span className="text-destructive ml-1">*</span>}
                </Label>
            )}
            {renderInput()}
            {description && !showError && (
                <p className="text-sm text-muted-foreground">{description}</p>
            )}
            {showError && (
                <p id={`${ schema.id }-error`} className="text-sm text-destructive" role="alert">
                    {fieldError}
                </p>
            )}
        </div>
    );
}

function FormRenderer({
    schema,
}: { schema: FormSchema }): React.ReactElement {
    const ctx = useRendererContext();
    const handleSubmitAction = useEventHandler(schema.events?.onSubmit);
    const submitLabel = useResolvedValue(schema.submitLabel);
    const cancelLabel = useResolvedValue(schema.cancelLabel);

    // Build Zod schema for validation
    const zodSchema = useMemo(
        () => buildFormSchema(schema.fields),
        [ schema.fields ]
    );

    // Get initial values from page state or field defaults
    // Note: we get state once during initialization, not reactively
    const initialValues = useMemo(
        () => getInitialFormValues(schema.fields, ctx.state.getState().state),
        [schema.fields]
    );

    // Create TanStack Form instance with zod standard schema validation
    const form = useForm({
        defaultValues: initialValues,
        validators:    {
            onChange: zodSchema,
            onBlur:   zodSchema,
        },
        onSubmit: async({
            value,
        }) => {
            // Sync all form values to page state based on bindTo
            for (const field of schema.fields) {
                if (field.bindTo && field.name in value) {
                    ctx.state.setState(field.bindTo, value[field.name]);
                }
            }

            // Execute onSubmit actions
            await handleSubmitAction(value);
        },
    });

    // Form context value
    const formContextValue = useMemo<FormContextValue>(() => ({
        form,
        formId:   schema.id,
        isInForm: true,
    }), [
        form,
        schema.id,
    ]);

    const layoutClasses = {
        vertical:   `space-y-4`,
        horizontal: `grid grid-cols-2 gap-4`,
        inline:     `flex flex-wrap gap-4 items-end`,
    };

    const handleFormSubmit = (e: React.FormEvent): void => {
        e.preventDefault();
        void form.handleSubmit();
    };

    const handleReset = (): void => {
        form.reset();
    };

    const {
        isSubmitting,
    } = form.state;

    return (
        <FormContext.Provider value={formContextValue}>
            <form
                id={schema.id}
                className={`${ layoutClasses[schema.layout ?? `vertical`] } ${ schema.className ?? `` }`}
                style={schema.style as React.CSSProperties}
                onSubmit={handleFormSubmit}
                noValidate
            >
                {schema.fields.map((field) => (
                    <FieldRenderer key={field.id} schema={field} />
                ))}
                <div className="flex gap-2">
                    <Button type="submit" disabled={isSubmitting}>
                        {isSubmitting && <LucideIcons.Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                        {submitLabel || `Submit`}
                    </Button>
                    {cancelLabel && (
                        <Button type="button" variant="outline" onClick={handleReset}>
                            {cancelLabel}
                        </Button>
                    )}
                    {schema.showReset && (
                        <Button type="button" variant="ghost" onClick={handleReset}>
                            Reset
                        </Button>
                    )}
                </div>
            </form>
        </FormContext.Provider>
    );
}

function TableRenderer({
    schema,
}: { schema: TableSchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);
    const handleRowClick = useEventHandler(schema.events?.onRowClick);
    const handleSortChange = useEventHandler(schema.events?.onSortChange);
    const handlePageChange = useEventHandler(schema.events?.onPageChange);
    const handleSelect = useEventHandler(schema.events?.onSelect);

    // Local state for sorting and pagination
    const [
        sortColumn,
        setSortColumn,
    ] = React.useState<string | null>(null);
    const [
        sortDirection,
        setSortDirection,
    ] = React.useState<`asc` | `desc`>(`asc`);
    const [
        currentPage,
        setCurrentPage,
    ] = React.useState(1);
    const [
        selectedRows,
        setSelectedRows,
    ] = React.useState<Set<string | number>>(new Set());

    const dataPath = schema.dataSource.startsWith(`state:`)
        ? schema.dataSource.slice(6)
        : schema.dataSource;
    let data = (getNestedValue(stateData, dataPath) ?? []) as Array<Record<string, unknown>>;

    const isLoading = schema.loading
        ? evaluateBooleanExpression(schema.loading, stateData)
        : false;

    // Apply sorting
    if (sortColumn && schema.sortable !== false) {
        data = [ ...data ].sort((a, b) => {
            const aVal = a[sortColumn];
            const bVal = b[sortColumn];

            if (aVal === bVal) {
                return 0;
            }
            if (aVal === null || aVal === undefined) {
                return 1;
            }
            if (bVal === null || bVal === undefined) {
                return -1;
            }

            let comparison = 0;
            if (typeof aVal === `string` && typeof bVal === `string`) {
                comparison = aVal.localeCompare(bVal);
            }
            else if (typeof aVal === `number` && typeof bVal === `number`) {
                comparison = aVal - bVal;
            }
            else {
                comparison = String(aVal).localeCompare(String(bVal));
            }

            return sortDirection === `asc` ? comparison : -comparison;
        });
    }

    // Get pagination config
    const paginationConfig = typeof schema.pagination === `object`
        ? schema.pagination
        : schema.pagination
            ? {
                pageSize: 10,
            }
            : null;
    const pageSize = paginationConfig?.pageSize ?? 10;
    const totalPages = Math.ceil(data.length / pageSize);
    const totalItems = data.length;

    // Apply pagination
    let paginatedData = data;
    if (paginationConfig) {
        const start = (currentPage - 1) * pageSize;
        paginatedData = data.slice(start, start + pageSize);
    }

    // Handle sort click
    const onSortClick = (columnKey: string): void => {
        const col = schema.columns.find((c) => c.key === columnKey);
        if (!col?.sortable && !schema.sortable) {
            return;
        }

        let newDirection: `asc` | `desc` = `asc`;
        if (sortColumn === columnKey) {
            newDirection = sortDirection === `asc` ? `desc` : `asc`;
        }

        setSortColumn(columnKey);
        setSortDirection(newDirection);

        void handleSortChange({
            column:    columnKey,
            direction: newDirection,
        });
    };

    // Handle page change
    const onPageChange = (page: number): void => {
        setCurrentPage(page);
        void handlePageChange({
            page,
            pageSize,
        });
    };

    // Handle row selection
    const onRowSelect = (rowKey: string | number, selected: boolean): void => {
        const newSelected = new Set(selectedRows);
        if (selected) {
            if (schema.selectable === `single`) {
                newSelected.clear();
            }
            newSelected.add(rowKey);
        }
        else {
            newSelected.delete(rowKey);
        }
        setSelectedRows(newSelected);

        void handleSelect({
            selected: Array.from(newSelected),
            row:      rowKey,
            action:   selected ? `select` : `deselect`,
        });
    };

    // Handle select all
    const onSelectAll = (selected: boolean): void => {
        if (selected) {
            const allKeys = paginatedData.map((row, index) => schema.rowKey ? row[schema.rowKey] : index
            );
            setSelectedRows(new Set(allKeys as Array<string | number>));
        }
        else {
            setSelectedRows(new Set());
        }
    };

    if (isLoading) {
        return (
            <div className="space-y-2">
                {[
                    1,
                    2,
                    3,
                ].map((i) => (
                    <Skeleton key={i} className="h-12 w-full" />
                ))}
            </div>
        );
    }

    if (data.length === 0) {
        const emptyText = useResolvedValue(schema.emptyText) || `No data`;
        return (
            <div className="text-center py-8 text-muted-foreground">
                {emptyText}
            </div>
        );
    }

    const isSelectable = schema.selectable === true || schema.selectable === `single` || schema.selectable === `multiple`;
    const allSelected = paginatedData.length > 0 && paginatedData.every((row, index) => {
        const key = schema.rowKey ? row[schema.rowKey] : index;
        return selectedRows.has(key as string | number);
    });

    return (
        <div className={`space-y-4 ${ schema.className ?? `` }`} style={schema.style as React.CSSProperties}>
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            {isSelectable && schema.selectable !== `single` && (
                                <TableHead className="w-12">
                                    <Checkbox
                                        checked={allSelected}
                                        onCheckedChange={onSelectAll}
                                    />
                                </TableHead>
                            )}
                            {isSelectable && schema.selectable === `single` && (
                                <TableHead className="w-12" />
                            )}
                            {schema.columns.map((col) => {
                                const isSortable = col.sortable !== false && (col.sortable || schema.sortable);
                                const isSorted = sortColumn === col.key;

                                return (
                                    <TableHead
                                        key={col.key}
                                        style={{
                                            width: col.width,
                                        }}
                                        className={`${ col.align ? `text-${ col.align }` : `` } ${ isSortable ? `cursor-pointer select-none hover:bg-muted/50` : `` }`}
                                        onClick={isSortable ? () => onSortClick(col.key) : undefined}
                                    >
                                        <div className="flex items-center gap-1">
                                            {useResolvedValue(col.label)}
                                            {isSortable && (
                                                <span className="ml-1">
                                                    {isSorted
                                                        ? sortDirection === `asc`
                                                            ? <LucideIcons.ChevronUp className="h-4 w-4" />
                                                            : <LucideIcons.ChevronDown className="h-4 w-4" />
                                                        : <LucideIcons.ChevronsUpDown className="h-4 w-4 opacity-50" />
                                                    }
                                                </span>
                                            )}
                                        </div>
                                    </TableHead>
                                );
                            })}
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {paginatedData.map((row, rowIndex) => {
                            const rowKey = schema.rowKey ? row[schema.rowKey] : rowIndex;
                            const isSelected = selectedRows.has(rowKey as string | number);

                            return (
                                <TableRow
                                    key={String(rowKey)}
                                    className={`${ schema.events?.onRowClick ? `cursor-pointer` : `` } ${ isSelected ? `bg-muted/50` : `` }`}
                                    onClick={() => {
                                        if (schema.events?.onRowClick) {
                                            void handleRowClick({
                                                row,
                                                index: rowIndex,
                                            });
                                        }
                                    }}
                                >
                                    {isSelectable && (
                                        <TableCell className="w-12">
                                            <Checkbox
                                                checked={isSelected}
                                                onCheckedChange={(checked) => onRowSelect(rowKey as string | number, Boolean(checked))}
                                                onClick={(e) => e.stopPropagation()}
                                            />
                                        </TableCell>
                                    )}
                                    {schema.columns.map((col) => (
                                        <TableCell
                                            key={col.key}
                                            className={col.align ? `text-${ col.align }` : undefined}
                                        >
                                            {col.render
? (
                                                <RendererContextValue.Provider value={{
                                                    ...ctx,
                                                    row,
                                                    index: rowIndex,
                                                }}>
                                                    <ComponentRenderer schema={col.render} />
                                                </RendererContextValue.Provider>
                                            )
: (
                                                String(row[col.key] ?? ``)
                                            )}
                                        </TableCell>
                                    ))}
                                </TableRow>
                            );
                        })}
                    </TableBody>
                </Table>
            </div>

            {/* Pagination */}
            {paginationConfig && totalPages > 1 && (
                <div className="flex items-center justify-between px-2">
                    {paginationConfig.showTotal !== false && (
                        <div className="text-sm text-muted-foreground">
                            Showing {(currentPage - 1) * pageSize + 1} to {Math.min(currentPage * pageSize, totalItems)} of {totalItems} items
                        </div>
                    )}
                    <div className="flex items-center gap-2">
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onPageChange(currentPage - 1)}
                            disabled={currentPage <= 1}
                        >
                            <LucideIcons.ChevronLeft className="h-4 w-4" />
                        </Button>
                        <div className="flex items-center gap-1">
                            {Array.from({
                                length: Math.min(totalPages, 5),
                            }, (_, i) => {
                                let pageNum: number;
                                if (totalPages <= 5) {
                                    pageNum = i + 1;
                                }
                                else if (currentPage <= 3) {
                                    pageNum = i + 1;
                                }
                                else if (currentPage >= totalPages - 2) {
                                    pageNum = totalPages - 4 + i;
                                }
                                else {
                                    pageNum = currentPage - 2 + i;
                                }
                                return (
                                    <Button
                                        key={pageNum}
                                        variant={currentPage === pageNum ? `default` : `outline`}
                                        size="sm"
                                        onClick={() => onPageChange(pageNum)}
                                        className="w-8 h-8 p-0"
                                    >
                                        {pageNum}
                                    </Button>
                                );
                            })}
                        </div>
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={() => onPageChange(currentPage + 1)}
                            disabled={currentPage >= totalPages}
                        >
                            <LucideIcons.ChevronRight className="h-4 w-4" />
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}

function CardRenderer({
    schema,
}: { schema: CardSchema }): React.ReactElement {
    const handleClick = useEventHandler(schema.events?.onClick);
    const title = useResolvedValue(schema.title);
    const subtitle = useResolvedValue(schema.subtitle);

    const handleOnClick = (): void => {
        void handleClick();
    };

    const hoverClass = schema.hoverable ? `hover:shadow-md transition-shadow cursor-pointer` : ``;
    const combinedClass = `${ hoverClass } ${ schema.className ?? `` }`.trim();

    return (
        <Card
            id={schema.id}
            className={combinedClass}
            style={schema.style as React.CSSProperties}
            onClick={schema.events?.onClick ? handleOnClick : undefined}
        >
            {(title || subtitle || schema.header) && (
                <CardHeader>
                    {schema.header
? (
                        <ComponentRenderer schema={schema.header} />
                    )
: (
                        <>
                            {title && <CardTitle>{title}</CardTitle>}
                            {subtitle && <CardDescription>{subtitle}</CardDescription>}
                        </>
                    )}
                </CardHeader>
            )}
            <CardContent>
                <ComponentRenderer schema={schema.content} />
            </CardContent>
            {schema.footer && (
                <CardFooter>
                    <ComponentRenderer schema={schema.footer} />
                </CardFooter>
            )}
        </Card>
    );
}

function ListRenderer({
    schema,
}: { schema: ListSchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);
    const handleRowClick = useEventHandler(schema.events?.onRowClick);

    const dataPath = schema.dataSource.startsWith(`state:`)
        ? schema.dataSource.slice(6)
        : schema.dataSource;
    const data = (getNestedValue(stateData, dataPath) ?? []) as Array<unknown>;

    const isLoading = schema.loading
        ? evaluateBooleanExpression(schema.loading, stateData)
        : false;

    if (isLoading) {
        return (
            <div className="space-y-2">
                {[
                    1,
                    2,
                    3,
                ].map((i) => (
                    <Skeleton key={i} className="h-16 w-full" />
                ))}
            </div>
        );
    }

    if (data.length === 0) {
        if (schema.emptyTemplate) {
            return <ComponentRenderer schema={schema.emptyTemplate} />;
        }
        const emptyText = useResolvedValue(schema.emptyText) || `No items`;
        return (
            <div className="text-center py-8 text-muted-foreground">
                {emptyText}
            </div>
        );
    }

    return (
        <div className={`space-y-2 ${ schema.className ?? `` }`} style={schema.style as React.CSSProperties}>
            {data.map((item, index) => (
                <div
                    key={index}
                    className={schema.events?.onRowClick ? `cursor-pointer` : undefined}
                    onClick={() => {
                        if (schema.events?.onRowClick) {
                            void handleRowClick({
                                item,
                                index,
                            });
                        }
                    }}
                >
                    <RendererContextValue.Provider value={{
                        ...ctx,
                        item,
                        index,
                    }}>
                        <ComponentRenderer schema={schema.itemTemplate} />
                    </RendererContextValue.Provider>
                </div>
            ))}
        </div>
    );
}

function ImageRenderer({
    schema,
}: { schema: ImageSchema }): React.ReactElement {
    const src = useResolvedValue(schema.src);
    const alt = useResolvedValue(schema.alt);

    return (
        <img
            id={schema.id}
            src={src}
            alt={alt}
            className={schema.className}
            style={{
                width:     schema.width,
                height:    schema.height,
                objectFit: schema.fit,
                ...schema.style,
            } as React.CSSProperties}
            loading={schema.loading}
        />
    );
}

function IconRenderer({
    schema,
}: { schema: IconSchema }): React.ReactElement {
    const handleClick = useEventHandler(schema.events?.onClick);
    const Icon = getIcon(schema.name);

    const sizeClasses: Record<string, string> = {
        xs: `h-3 w-3`,
        sm: `h-4 w-4`,
        md: `h-5 w-5`,
        lg: `h-6 w-6`,
        xl: `h-8 w-8`,
    };

    if (!Icon) {
        return <span className="text-muted-foreground">[{schema.name}]</span>;
    }

    const colorStyle = schema.color
? {
    color: schema.color,
}
: {};
    const iconElement = (
        <span style={colorStyle}>
            <Icon className={`${ sizeClasses[schema.size ?? `md`] } ${ schema.className ?? `` }`} />
        </span>
    );

    const handleOnClick = (): void => {
        void handleClick();
    };

    if (schema.events?.onClick) {
        return (
            <span
                onClick={handleOnClick}
                style={schema.style as React.CSSProperties}
                className="cursor-pointer"
            >
                {iconElement}
            </span>
        );
    }

    return iconElement;
}

function LinkRenderer({
    schema,
}: { schema: LinkSchema }): React.ReactElement {
    const href = useResolvedValue(schema.href);
    const text = useResolvedValue(schema.text);
    const Icon = schema.icon ? getIcon(schema.icon) : null;

    return (
        <a
            id={schema.id}
            href={href}
            target={schema.external ? `_blank` : undefined}
            rel={schema.external ? `noopener noreferrer` : undefined}
            className={`text-primary hover:underline inline-flex items-center gap-1 ${ schema.className ?? `` }`}
            style={schema.style as React.CSSProperties}
        >
            {Icon && <Icon className="h-4 w-4" />}
            {text}
            {schema.external && <LucideIcons.ExternalLink className="h-3 w-3" />}
        </a>
    );
}

function BadgeRenderer({
    schema,
}: { schema: BadgeSchema }): React.ReactElement {
    const text = useResolvedValue(schema.text);

    return (
        <Badge
            id={schema.id}
            variant={schema.variant}
            className={schema.className}
            style={schema.style as React.CSSProperties}
        >
            {text}
        </Badge>
    );
}

function AlertRenderer({
    schema,
}: { schema: AlertSchema }): React.ReactElement {
    const title = useResolvedValue(schema.title);
    const message = useResolvedValue(schema.message);
    const Icon = schema.icon ? getIcon(schema.icon) : null;

    return (
        <Alert
            id={schema.id}
            variant={schema.variant}
            className={schema.className}
            style={schema.style as React.CSSProperties}
        >
            {Icon && <Icon className="h-4 w-4" />}
            {title && <AlertTitle>{title}</AlertTitle>}
            <AlertDescription>{message}</AlertDescription>
        </Alert>
    );
}

function ProgressRenderer({
    schema,
}: { schema: ProgressSchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);

    const value = typeof schema.value === `string`
        ? Number(interpolateExpression(schema.value, stateData))
        : schema.value;
    const max = schema.max ?? 100;

    return (
        <div className={`space-y-1 ${ schema.className ?? `` }`} style={schema.style as React.CSSProperties}>
            <Progress id={schema.id} value={(value / max) * 100} />
            {schema.showLabel && (
                <p className="text-sm text-muted-foreground text-right">{value}/{max}</p>
            )}
        </div>
    );
}

function TabsRenderer({
    schema,
}: { schema: TabsSchema }): React.ReactElement {
    return (
        <Tabs
            id={schema.id}
            defaultValue={schema.defaultTab ?? schema.items[0]?.key}
            orientation={schema.orientation}
            className={schema.className}
            style={schema.style as React.CSSProperties}
        >
            <TabsList>
                {schema.items.map((item) => (
                    <TabsTrigger key={item.key} value={item.key}>
                        {item.icon && (() => {
                            const Icon = getIcon(item.icon);
                            return Icon ? <Icon className="mr-2 h-4 w-4" /> : null;
                        })()}
                        {useResolvedValue(item.label)}
                    </TabsTrigger>
                ))}
            </TabsList>
            {schema.items.map((item) => (
                <TabsContent key={item.key} value={item.key}>
                    <ComponentRenderer schema={item.content} />
                </TabsContent>
            ))}
        </Tabs>
    );
}

function AccordionRenderer({
    schema,
}: { schema: AccordionSchema }): React.ReactElement {
    const accordionType = schema.type_ ?? `single`;

    if (accordionType === `multiple`) {
        return (
            <Accordion
                id={schema.id}
                type="multiple"
                defaultValue={schema.defaultOpen}
                className={schema.className}
                style={schema.style as React.CSSProperties}
            >
                {schema.items.map((item) => (
                    <AccordionItem key={item.key} value={item.key}>
                        <AccordionTrigger>{useResolvedValue(item.title)}</AccordionTrigger>
                        <AccordionContent>
                            <ComponentRenderer schema={item.content} />
                        </AccordionContent>
                    </AccordionItem>
                ))}
            </Accordion>
        );
    }

    return (
        <Accordion
            id={schema.id}
            type="single"
            collapsible={schema.collapsible}
            defaultValue={schema.defaultOpen?.[0]}
            className={schema.className}
            style={schema.style as React.CSSProperties}
        >
            {schema.items.map((item) => (
                <AccordionItem key={item.key} value={item.key}>
                    <AccordionTrigger>{useResolvedValue(item.title)}</AccordionTrigger>
                    <AccordionContent>
                        <ComponentRenderer schema={item.content} />
                    </AccordionContent>
                </AccordionItem>
            ))}
        </Accordion>
    );
}

function ModalRenderer({
    schema,
}: { schema: ModalSchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);

    const dialogState = getNestedValue(stateData, `__dialogs.${ schema.id }`) as { open?: boolean } | undefined;
    const isOpen = dialogState?.open ?? false;

    const title = useResolvedValue(schema.title);
    const description = useResolvedValue(schema.description);

    return (
        <Dialog
            open={isOpen}
            onOpenChange={(open) => {
                ctx.state.setState(`__dialogs.${ schema.id }`, {
                    open,
                });
            }}
        >
            <DialogContent className={schema.className} style={schema.style as React.CSSProperties}>
                {(title || description) && (
                    <DialogHeader>
                        {title && <DialogTitle>{title}</DialogTitle>}
                        {description && <DialogDescription>{description}</DialogDescription>}
                    </DialogHeader>
                )}
                <ComponentRenderer schema={schema.content} />
                {schema.footer && (
                    <DialogFooter>
                        <ComponentRenderer schema={schema.footer} />
                    </DialogFooter>
                )}
            </DialogContent>
        </Dialog>
    );
}

function DropdownItemRenderer({
    item,
}: { item: DropdownSchema[`items`][number] }): React.ReactElement | null {
    const handleClick = useEventHandler(item.events?.onClick);
    const Icon = item.icon ? getIcon(item.icon) : null;
    const label = useResolvedValue(item.label);

    const handleOnClick = (): void => {
        void handleClick();
    };

    if (item.separator) {
        return <DropdownMenuSeparator />;
    }

    return (
        <DropdownMenuItem
            key={item.key}
            disabled={item.disabled ? evaluateBooleanExpression(item.disabled, {}) : false}
            className={item.danger ? `text-destructive focus:text-destructive` : undefined}
            onClick={handleOnClick}
        >
            {Icon && <Icon className="mr-2 h-4 w-4" />}
            {label}
        </DropdownMenuItem>
    );
}

function DropdownRenderer({
    schema,
}: { schema: DropdownSchema }): React.ReactElement {
    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <div>
                    <ComponentRenderer schema={schema.trigger} />
                </div>
            </DropdownMenuTrigger>
            <DropdownMenuContent align={schema.align}>
                {schema.items.map((item, i) => (
                    <DropdownItemRenderer key={item.key ?? i} item={item} />
                ))}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

function TooltipRenderer({
    schema,
}: { schema: TooltipSchema }): React.ReactElement {
    const content = useResolvedValue(schema.content);

    return (
        <Tooltip delayDuration={schema.delayMs}>
            <TooltipTrigger asChild>
                <div>
                    <ComponentRenderer schema={schema.children} />
                </div>
            </TooltipTrigger>
            <TooltipContent side={schema.side}>
                {content}
            </TooltipContent>
        </Tooltip>
    );
}

function GridRenderer({
    schema,
}: { schema: GridSchema }): React.ReactElement {
    const columns = typeof schema.columns === `number`
        ? schema.columns
        : schema.columns.md ?? 3;

    return (
        <div
            id={schema.id}
            className={`grid ${ schema.className ?? `` }`}
            style={{
                gridTemplateColumns: `repeat(${ columns }, minmax(0, 1fr))`,
                gap:                 schema.gap ?? `1rem`,
                ...schema.style,
            } as React.CSSProperties}
        >
            {schema.children.map((child, i) => (
                <ComponentRenderer key={child.id ?? i} schema={child} />
            ))}
        </div>
    );
}

function FlexRenderer({
    schema,
}: { schema: FlexSchema }): React.ReactElement {
    const justifyMap: Record<string, string> = {
        start:   `flex-start`,
        end:     `flex-end`,
        center:  `center`,
        between: `space-between`,
        around:  `space-around`,
        evenly:  `space-evenly`,
    };

    const alignMap: Record<string, string> = {
        start:    `flex-start`,
        end:      `flex-end`,
        center:   `center`,
        stretch:  `stretch`,
        baseline: `baseline`,
    };

    return (
        <div
            id={schema.id}
            className={schema.className}
            style={{
                display:        `flex`,
                flexDirection:  schema.direction ?? `row`,
                justifyContent: justifyMap[schema.justify ?? `start`],
                alignItems:     alignMap[schema.align ?? `stretch`],
                gap:            schema.gap ?? `0.5rem`,
                flexWrap:       schema.wrap ? `wrap` : undefined,
                ...schema.style,
            } as React.CSSProperties}
        >
            {schema.children.map((child, i) => (
                <ComponentRenderer key={child.id ?? i} schema={child} />
            ))}
        </div>
    );
}

function SpacerRenderer({
    schema,
}: { schema: SpacerSchema }): React.ReactElement {
    const sizeMap: Record<string, string> = {
        xs: `0.25rem`,
        sm: `0.5rem`,
        md: `1rem`,
        lg: `1.5rem`,
        xl: `2rem`,
    };

    return <div style={{
        height: sizeMap[schema.size],
        width:  `100%`,
    }} />;
}

function DividerRenderer({
    schema,
}: { schema: DividerSchema }): React.ReactElement {
    if (schema.label) {
        return (
            <div className={`flex items-center gap-4 ${ schema.className ?? `` }`}>
                <Separator className="flex-1" />
                <span className="text-sm text-muted-foreground">{useResolvedValue(schema.label)}</span>
                <Separator className="flex-1" />
            </div>
        );
    }

    return (
        <Separator
            orientation={schema.orientation}
            className={schema.className}
            style={schema.style as React.CSSProperties}
        />
    );
}

function SkeletonRenderer({
    schema,
}: { schema: SkeletonSchema }): React.ReactElement {
    const variantClasses: Record<string, string> = {
        text:        `h-4`,
        circular:    `rounded-full`,
        rectangular: ``,
    };

    return (
        <Skeleton
            className={`${ variantClasses[schema.variant ?? `rectangular`] } ${ schema.className ?? `` }`}
            style={{
                width:  schema.width,
                height: schema.height,
                ...schema.style,
            } as React.CSSProperties}
        />
    );
}

function AvatarRenderer({
    schema,
}: { schema: AvatarSchema }): React.ReactElement {
    const src = useResolvedValue(schema.src);
    const alt = useResolvedValue(schema.alt);
    const fallback = useResolvedValue(schema.fallback);

    const sizeClasses: Record<string, string> = {
        xs: `h-6 w-6`,
        sm: `h-8 w-8`,
        md: `h-10 w-10`,
        lg: `h-12 w-12`,
        xl: `h-16 w-16`,
    };

    return (
        <Avatar className={`${ sizeClasses[schema.size ?? `md`] } ${ schema.className ?? `` }`}>
            {src && <AvatarImage src={src} alt={alt} />}
            <AvatarFallback>{fallback || alt?.charAt(0) || `?`}</AvatarFallback>
        </Avatar>
    );
}

function BreadcrumbRenderer({
    schema,
}: { schema: BreadcrumbSchema }): React.ReactElement {
    return (
        <Breadcrumb className={schema.className} style={schema.style as React.CSSProperties}>
            <BreadcrumbList>
                {schema.items.map((item, i) => {
                    const label = useResolvedValue(item.label);
                    const href = useResolvedValue(item.href);
                    const Icon = item.icon ? getIcon(item.icon) : null;
                    const isLast = i === schema.items.length - 1;

                    return (
                        <React.Fragment key={i}>
                            <BreadcrumbItem>
                                {isLast
? (
                                    <BreadcrumbPage className="flex items-center gap-1">
                                        {Icon && <Icon className="h-4 w-4" />}
                                        {label}
                                    </BreadcrumbPage>
                                )
: (
                                    <BreadcrumbLink href={href} className="flex items-center gap-1">
                                        {Icon && <Icon className="h-4 w-4" />}
                                        {label}
                                    </BreadcrumbLink>
                                )}
                            </BreadcrumbItem>
                            {!isLast && <BreadcrumbSeparator />}
                        </React.Fragment>
                    );
                })}
            </BreadcrumbList>
        </Breadcrumb>
    );
}

function StatCardRenderer({
    schema,
}: { schema: StatCardSchema }): React.ReactElement {
    const title = useResolvedValue(schema.title);
    const value = useResolvedValue(schema.value);
    const change = useResolvedValue(schema.change);
    const description = useResolvedValue(schema.description);
    const Icon = schema.icon ? getIcon(schema.icon) : null;

    const getChangeColor = (): string => {
        if (schema.changeType === `increase`) {
            return `text-green-600`;
        }
        if (schema.changeType === `decrease`) {
            return `text-red-600`;
        }
        return `text-muted-foreground`;
    };
    const changeColor = getChangeColor();

    return (
        <Card className={schema.className} style={schema.style as React.CSSProperties}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{title}</CardTitle>
                {Icon && <Icon className="h-4 w-4 text-muted-foreground" />}
            </CardHeader>
            <CardContent>
                <div className="text-2xl font-bold">{value}</div>
                {(change || description) && (
                    <p className={`text-xs ${ change ? changeColor : `text-muted-foreground` }`}>
                        {change || description}
                    </p>
                )}
            </CardContent>
        </Card>
    );
}

function EmptyStateRenderer({
    schema,
}: { schema: EmptyStateSchema }): React.ReactElement {
    const title = useResolvedValue(schema.title);
    const description = useResolvedValue(schema.description);
    const Icon = schema.icon ? getIcon(schema.icon) : null;

    return (
        <div className={`flex flex-col items-center justify-center py-12 text-center ${ schema.className ?? `` }`}>
            {Icon && <Icon className="h-12 w-12 text-muted-foreground mb-4" />}
            <h3 className="text-lg font-medium">{title}</h3>
            {description && <p className="text-muted-foreground mt-1">{description}</p>}
            {schema.action && (
                <div className="mt-4">
                    <ButtonRenderer schema={schema.action} />
                </div>
            )}
        </div>
    );
}

function LoadingOverlayRenderer({
    schema,
}: { schema: LoadingOverlaySchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);
    const isLoading = evaluateBooleanExpression(schema.loading, stateData);
    const text = useResolvedValue(schema.text);

    return (
        <div className={`relative ${ schema.className ?? `` }`}>
            <ComponentRenderer schema={schema.children} />
            {isLoading && (
                <div className="absolute inset-0 bg-background/80 flex items-center justify-center">
                    <div className="flex flex-col items-center gap-2">
                        <LucideIcons.Loader2 className="h-8 w-8 animate-spin text-primary" />
                        {text && <p className="text-sm text-muted-foreground">{text}</p>}
                    </div>
                </div>
            )}
        </div>
    );
}

function ConditionalRenderer({
    schema,
}: { schema: ConditionalSchema }): React.ReactElement | null {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);
    const is_condition_met = evaluateBooleanExpression(schema.condition, stateData);

    if (is_condition_met) {
        return <ComponentRenderer schema={schema.then} />;
    }

    if (schema.else) {
        return <ComponentRenderer schema={schema.else} />;
    }

    return null;
}

function LoopRenderer({
    schema,
}: { schema: LoopSchema }): React.ReactElement {
    const ctx = useRendererContext();

    const stateData = ctx.state((s) => s.state);

    const dataPath = schema.dataSource.startsWith(`state:`)
        ? schema.dataSource.slice(6)
        : schema.dataSource;
    const data = (getNestedValue(stateData, dataPath) ?? []) as Array<unknown>;

    if (data.length === 0 && schema.emptyTemplate) {
        return <ComponentRenderer schema={schema.emptyTemplate} />;
    }

    return (
        <>
            {data.map((item, index) => (
                <RendererContextValue.Provider
                    key={index}
                    value={{
                        ...ctx,
                        item,
                        index,
                    }}
                >
                    <ComponentRenderer schema={schema.template} />
                </RendererContextValue.Provider>
            ))}
        </>
    );
}

function SectionRenderer({
    schema,
}: { schema: SectionSchema }): React.ReactElement {
    const [
        isCollapsed,
        setIsCollapsed,
    ] = React.useState(schema.defaultCollapsed ?? false);
    const title = useResolvedValue(schema.title);
    const description = useResolvedValue(schema.description);

    return (
        <section
            id={schema.id}
            className={`space-y-4 ${ schema.className ?? `` }`}
            style={schema.style as React.CSSProperties}
        >
            {(title || description) && (
                <div className="flex items-center justify-between">
                    <div>
                        {title && <h2 className="text-xl font-semibold">{title}</h2>}
                        {description && <p className="text-muted-foreground">{description}</p>}
                    </div>
                    {schema.collapsible && (
                        <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setIsCollapsed(!isCollapsed)}
                        >
                            {isCollapsed
? (
                                <LucideIcons.ChevronDown className="h-4 w-4" />
                            )
: (
                                <LucideIcons.ChevronUp className="h-4 w-4" />
                            )}
                        </Button>
                    )}
                </div>
            )}
            {!isCollapsed && schema.children.map((child, i) => (
                <ComponentRenderer key={child.id ?? i} schema={child} />
            ))}
        </section>
    );
}

function PageHeaderRenderer({
    schema,
}: { schema: PageHeaderSchema }): React.ReactElement {
    const title = useResolvedValue(schema.title);
    const subtitle = useResolvedValue(schema.subtitle);
    const backLink = useResolvedValue(schema.backLink);
    const navigate = useNavigate();

    return (
        <div className={`space-y-4 ${ schema.className ?? `` }`} style={schema.style as React.CSSProperties}>
            {schema.breadcrumb && <BreadcrumbRenderer schema={{
                type:  `Breadcrumb`,
                items: schema.breadcrumb,
            }} />}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                    {backLink && (
                        <Button variant="ghost" size="sm" onClick={async() => navigate(backLink)}>
                            <LucideIcons.ArrowLeft className="h-4 w-4 mr-2" />
                            Back
                        </Button>
                    )}
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
                        {subtitle && <p className="text-muted-foreground">{subtitle}</p>}
                    </div>
                </div>
                {schema.actions && schema.actions.length > 0 && (
                    <div className="flex gap-2">
                        {schema.actions.map((action, i) => (
                            <ButtonRenderer key={i} schema={action} />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

function DataDisplayRenderer({
    schema,
}: { schema: DataDisplaySchema }): React.ReactElement {
    const label = useResolvedValue(schema.label);
    const value = useResolvedValue(schema.value);
    const containerClass = `flex flex-col space-y-1 ${ schema.className ?? `` }`;

    return (
        <div className={containerClass} style={schema.style as React.CSSProperties}>
            <span className="text-sm text-muted-foreground">{label}</span>
            <div className="flex items-center gap-2">
                {schema.prefix && <ComponentRenderer schema={schema.prefix} />}
                <span className="font-medium">{value}</span>
                {schema.suffix && <ComponentRenderer schema={schema.suffix} />}
                {schema.copyable && (
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                            void navigator.clipboard.writeText(value);
                        }}
                    >
                        <LucideIcons.Copy className="h-4 w-4" />
                    </Button>
                )}
            </div>
        </div>
    );
}

export {
    ComponentRenderer,
    useRendererContext,
    useEventHandler,
    useResolvedValue,
    getIcon
};
